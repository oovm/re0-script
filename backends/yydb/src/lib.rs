//! YYDB public facade — Rust embed `Connection` with **VOS as the only DDL and
//! operation language** (typed construction / collection methods; not SQL).
//!
//! **Embedded host:** Rust only. Register native [UDFs](udf) on
//! [`Connection`]. Other languages should use the `yydb-client` crate against
//! `yydb serve`, not this embeddable library.
//!
//! Durable layout is still **one primary `.yydb` file**. Optional journal
//! sidecars `{path}-wal` / `{path}-shm` appear when
//! [`JournalMode::Wal`](journal::JournalMode::Wal) is enabled. Binary payloads
//! use the unified [`objects`] CAS (`objects/hash-2/*.bytes`) with optional
//! runtime hot/cold tiering — not a `.yydb-vec` sidecar.
//!
//! ```rust,no_run
//! use yydb::{Connection, OpenFlags, Result, Value};
//!
//! fn main() -> Result<()> {
//!     let conn = Connection::open_with_flags("app.yydb", OpenFlags::wal())?;
//!     conn.ensure_schema(1, "table Project { @@id: uuid, title: utf8 }")?;
//!     conn.create_scalar("double", 1, |args| match args {
//!         [Value::I64(n)] => Ok(Value::I64(n * 2)),
//!         _ => Err(yydb::Error::Udf {
//!             name: "double".into(),
//!             message: "expected i64".into(),
//!         }),
//!     })?;
//!     conn.checkpoint()?;
//!     Ok(())
//! }
//! ```

#![deny(missing_docs)]

/// Shared types (`Error`, `Result`, `Value`, …).
pub mod types {
    pub use yydb_types::*;
}

/// Journal modes and `-wal` / `-shm` sidecar helpers.
pub mod journal;

/// Unified `objects/hash-2/*.bytes` CAS + hot/cold tiering.
pub mod objects;

/// Shared VOS schema contract (`vos` git @ `dev`).
pub mod schema;

/// Table-row storage validated by the VOS catalog.
pub mod table;

/// Interim secondary-index keys for unique fields.
pub mod index;

/// VOS operation-language surface (object / method style; executor pending).
pub mod query;

/// Fixed-size page I/O for the coming B+Tree layout.
pub mod pager;

/// B+Tree leaf page layout (pager-backed; not yet the Connection catalog).
pub mod btree;

/// Paged-era catalog pages (schema/catalog storage on `.yydb` pages).
pub mod catalog_page;

/// Paged-era KV records on B+Tree pages.
pub mod paged_store;

/// Durable field identity (`FieldId` + virtual slots) for DDL / macro IR.
///
/// Host runtime catalog — **not** part of `vos-ast` syntax.
pub mod schema_catalog;

/// Durable macro IR (virtual-slot bound) and lowering.
pub mod macros;

/// DDL session (`begin_ddl_session` / `Database.xxx` host path).
pub mod ddl;

/// Query / data sessions with `DdlRevision` freshness (`VOS-SESSION-STALE`).
pub mod session;

/// Process-local exclusive write gate (`Error::Busy` on contention).
pub mod concurrency;

/// Rust scalar UDF traits and registration helpers.
pub mod udf;

/// YY wire protocol (`YYDB`|`YYDS` + version digits `0000`…).
pub mod wire;

pub use ddl::DdlSession;
pub use journal::{CreateLayout, JournalMode, OpenFlags};
pub use objects::ObjectStore;
pub use session::{DataSession, PreparedPlan, QuerySession};
pub use table::Row;
pub use udf::ScalarUdf;
pub use yydb_types::{
    ChunkManifest, Error, HashAlgo, ObjectKind, ObjectRef, Result, SchemaVersion, Tier, Value,
    Vector, DEFAULT_CHUNK_SIZE, INLINE_BYTES_MAX,
};

/// VOS schema language facade (`git+https://github.com/voml/vos-language.git?branch=dev`).
pub use vos;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use journal::{
    append_snapshot_frame, ensure_wal_sidecars, last_complete_wal_payload, remove_wal_sidecars,
    replay_wal_snapshots, shm_path, truncate_wal, wal_frame_count, wal_path,
};
use pager::{Pager, PAGER_MAGIC};
use udf::{ClosureUdf, RegisteredUdf, ScalarFn};

const MAGIC: &[u8] = b"YYDB\x01";

#[derive(Debug, Default, Clone)]
struct State {
    schema: Option<SchemaVersion>,
    /// Durable field-identity catalog (marker `2` in the file format).
    schema_catalog: Option<schema_catalog::SchemaCatalog>,
    records: BTreeMap<String, Vec<u8>>,
}

/// On-disk layout era for a file-backed connection.
#[derive(Debug)]
enum FileLayout {
    /// Whole-file snapshot (`YYDB\x01`).
    Snapshot,
    /// Fixed-size pages (`YYDB\x02`); catalog on catalog pages.
    Paged(Mutex<Pager>),
}

enum Backend {
    File {
        path: PathBuf,
        journal_mode: Mutex<JournalMode>,
        layout: FileLayout,
    },
    Memory {
        state: Mutex<State>,
    },
}

/// A connection to a YYDB database (file-backed or in-memory).
///
/// Embedded `Connection` handle. DDL and query language are **VOS**.
/// UDFs are process-local and are **not** persisted in the `.yydb` file.
/// Binary payloads use [`ObjectStore`] (`objects/hash-2/*.bytes`).
///
/// File-backed connections share a **process-local exclusive write lease** per
/// path and block peer committed-reads while that lease is held
/// ([`Error::Busy`]). Cross-process writers use `{db}-write.lock`. This is not
/// MVCC — see `documentation/concurrency.md`.
pub struct Connection {
    backend: Backend,
    udfs: Mutex<BTreeMap<String, RegisteredUdf>>,
    objects: ObjectStore,
    /// Open transaction draft, if any (single-writer, connection-local).
    txn: Mutex<Option<State>>,
    /// `DdlRevision` captured at [`Self::begin`] (data txn fence).
    txn_ddl_revision: Mutex<Option<u64>>,
    /// Open DDL session draft catalog, if any.
    ddl_draft: Mutex<Option<schema_catalog::SchemaCatalog>>,
    /// Stable id for the process-local write gate.
    conn_id: u64,
    /// Shared gate for this file path (`None` for private in-memory DBs).
    write_gate: Option<Arc<concurrency::WriteGate>>,
    /// True while this connection holds a long write lease (data txn or DDL).
    write_lease: Mutex<bool>,
}

impl Connection {
    /// Open (or create) a database at `path` with delete journal mode.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_flags(path, OpenFlags::new())
    }

    /// Open (or create) a database with explicit [`OpenFlags`].
    ///
    /// When `path` is missing, creates a paged (`YYDB\x02`) file by default, or
    /// a snapshot (`YYDB\x01`) file when [`OpenFlags::create_snapshot`] (or
    /// related) is used. Existing files open by magic.
    pub fn open_with_flags(path: impl AsRef<Path>, flags: OpenFlags) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let layout = if path.exists() {
            open_file_layout(&path)?
        } else {
            match flags.create_layout {
                CreateLayout::Snapshot => {
                    write_main(&path, &State::default())?;
                    FileLayout::Snapshot
                }
                CreateLayout::Paged => {
                    let mut pager = Pager::create(&path)?;
                    catalog_page::init_empty(&mut pager)?;
                    FileLayout::Paged(Mutex::new(pager))
                }
            }
        };
        prepare_journal_sidecars(&path, flags.journal_mode, &layout)?;
        let connection = Self {
            backend: Backend::File {
                path: path.clone(),
                journal_mode: Mutex::new(flags.journal_mode),
                layout,
            },
            udfs: Mutex::new(BTreeMap::new()),
            objects: ObjectStore::open_beside_db(&path)?,
            txn: Mutex::new(None),
            txn_ddl_revision: Mutex::new(None),
            ddl_draft: Mutex::new(None),
            conn_id: concurrency::next_conn_id(),
            write_gate: Some(concurrency::gate_for_path(&path)),
            write_lease: Mutex::new(false),
        };
        // Force recovery path once so a leftover WAL is applied.
        let _ = connection.read_committed_state()?;
        Ok(connection)
    }

    /// Create a new **paged**-era database at `path` (`YYDB\x02`).
    ///
    /// Errors if `path` already exists. For create-on-missing, prefer
    /// [`Self::open`] / [`OpenFlags::new`] (paged is the default) or
    /// [`OpenFlags::create_paged`].
    pub fn create_paged(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if path.exists() {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("{} already exists", path.display()),
            )));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut pager = Pager::create(&path)?;
        catalog_page::init_empty(&mut pager)?;
        let connection = Self {
            backend: Backend::File {
                path: path.clone(),
                journal_mode: Mutex::new(JournalMode::Delete),
                layout: FileLayout::Paged(Mutex::new(pager)),
            },
            udfs: Mutex::new(BTreeMap::new()),
            objects: ObjectStore::open_beside_db(&path)?,
            txn: Mutex::new(None),
            txn_ddl_revision: Mutex::new(None),
            ddl_draft: Mutex::new(None),
            conn_id: concurrency::next_conn_id(),
            write_gate: Some(concurrency::gate_for_path(&path)),
            write_lease: Mutex::new(false),
        };
        Ok(connection)
    }

    /// True when this connection uses the paged `.yydb` file layout.
    pub fn is_paged(&self) -> bool {
        matches!(
            &self.backend,
            Backend::File {
                layout: FileLayout::Paged(_),
                ..
            }
        )
    }

    /// Open a private in-memory database (useful for tests).
    pub fn open_in_memory() -> Result<Self> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("yydb-mem-objects-{nonce}"));
        Ok(Self {
            backend: Backend::Memory {
                state: Mutex::new(State::default()),
            },
            udfs: Mutex::new(BTreeMap::new()),
            objects: ObjectStore::open_in_memory_root(root)?,
            txn: Mutex::new(None),
            txn_ddl_revision: Mutex::new(None),
            ddl_draft: Mutex::new(None),
            conn_id: concurrency::next_conn_id(),
            write_gate: None,
            write_lease: Mutex::new(false),
        })
    }

    /// Content-addressed object store (`<db>.objects/objects/…`).
    pub fn objects(&self) -> &ObjectStore {
        &self.objects
    }

    /// Filesystem path for file-backed connections; `None` for in-memory.
    pub fn path(&self) -> Option<&Path> {
        match &self.backend {
            Backend::File { path, .. } => Some(path.as_path()),
            Backend::Memory { .. } => None,
        }
    }

    /// Path of the `-wal` sidecar when file-backed.
    pub fn wal_path(&self) -> Option<PathBuf> {
        self.path().map(wal_path)
    }

    /// Path of the `-shm` sidecar when file-backed.
    pub fn shm_path(&self) -> Option<PathBuf> {
        self.path().map(shm_path)
    }

    /// Current journal mode (`delete` or `wal`). In-memory is always `delete`.
    pub fn journal_mode(&self) -> JournalMode {
        match &self.backend {
            Backend::File { journal_mode, .. } => *journal_mode
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
            Backend::Memory { .. } => JournalMode::Delete,
        }
    }

    /// Switch journal mode. Enabling WAL creates sidecars; disabling WAL
    /// checkpoints then removes `{db}-wal` / `{db}-shm`.
    pub fn set_journal_mode(&self, mode: JournalMode) -> Result<()> {
        let Backend::File {
            path,
            journal_mode,
            layout,
        } = &self.backend
        else {
            return Ok(());
        };
        let mut slot = journal_mode
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *slot == mode {
            return Ok(());
        }
        match mode {
            JournalMode::Wal => {
                ensure_wal_sidecars(path)?;
            }
            JournalMode::Delete => {
                // Fold recovered state into the main file before dropping WAL.
                // Do not call `write_committed_state` here — slot still says Wal.
                let state = self.read_committed_state()?;
                self.with_write_lease(|| match layout {
                    FileLayout::Snapshot => write_main(path, &state),
                    FileLayout::Paged(pager) => {
                        let mut guard = pager
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        write_paged_state(&mut guard, &state)
                    }
                })?;
                remove_wal_sidecars(path)?;
            }
        }
        *slot = mode;
        Ok(())
    }

    /// Fold WAL frames into the main file and truncate `-wal` / reset `-shm`.
    ///
    /// Order is intentional for crash safety: write the main file first, then
    /// truncate sidecars. A crash after the main write leaves main already at
    /// the folded state while `-wal` may still hold the same type-1 frames;
    /// reopen applies **last complete frame wins** and remains consistent
    /// (never a half-applied mix). See `documentation/file-format.md`.
    ///
    /// No-op when not in WAL mode or when there is nothing to fold.
    pub fn checkpoint(&self) -> Result<()> {
        let Backend::File {
            path,
            journal_mode,
            layout,
        } = &self.backend
        else {
            return Ok(());
        };
        let mode = *journal_mode
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if mode != JournalMode::Wal {
            return Ok(());
        }
        let state = self.read_committed_state()?;
        self.with_write_lease(|| match layout {
            FileLayout::Snapshot => write_main(path, &state),
            FileLayout::Paged(pager) => {
                let mut guard = pager
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                write_paged_state(&mut guard, &state)
            }
        })?;
        truncate_wal(path)?;
        Ok(())
    }

    /// Number of committed frames recorded in `-shm` (0 if absent).
    pub fn wal_frame_count(&self) -> Result<u32> {
        match &self.backend {
            Backend::File { path, .. } => wal_frame_count(path),
            Backend::Memory { .. } => Ok(0),
        }
    }

    /// Current database-truth schema, if any.
    pub fn schema(&self) -> Result<Option<SchemaVersion>> {
        Ok(self.read_state()?.schema)
    }

    /// Committed field-identity catalog (`FieldId` / virtual slots), if any.
    pub fn field_catalog(&self) -> Result<Option<schema_catalog::SchemaCatalog>> {
        Ok(self.read_state()?.schema_catalog.clone())
    }

    /// Current committed `DdlRevision` (`0` when no catalog).
    pub fn ddl_revision(&self) -> Result<u64> {
        Ok(self.field_catalog()?.map(|c| c.revisions.ddl).unwrap_or(0))
    }

    /// Begin a DDL write session (`DDL_WRITE`). Conflicts with an open data txn.
    pub fn begin_ddl_session(&self) -> Result<DdlSession<'_>> {
        DdlSession::open(self)
    }

    /// Begin a query session (`DATA_READ` / `DDL_READ`).
    ///
    /// Captures the current `DdlRevision`; later DDL commits make
    /// [`QuerySession::query`] fail with `VOS-SESSION-STALE`.
    pub fn begin_query_session(&self) -> Result<QuerySession<'_>> {
        QuerySession::open(self)
    }

    /// Begin a data session (`DATA_WRITE` allowed; not `DDL_WRITE`).
    pub fn begin_data_session(&self) -> Result<DataSession<'_>> {
        DataSession::open(self)
    }

    /// Parsed VOS catalog for the stored schema document, if any.
    ///
    /// Empty init placeholders yield `Ok(None)`. Language errors surface as
    /// [`Error::Schema`].
    pub fn parsed_schema(&self) -> Result<Option<vos::ast::Document>> {
        match self.schema()? {
            None => Ok(None),
            Some(schema) if schema::is_empty_document(&schema.document) => Ok(None),
            Some(schema) => Ok(Some(schema::parse_document(&schema.document)?)),
        }
    }

    /// Number of stored key/value records.
    pub fn record_count(&self) -> Result<usize> {
        Ok(self.read_state()?.records.len())
    }

    /// Stores the database truth schema when empty and rejects a mismatched
    /// version thereafter. `document` is **VOS** source (shared with
    /// `vos` git @ `dev`); it is validated before persistence.
    ///
    /// Changing an already-stored schema version is **not** done here — use
    /// [`Self::migrate_schema`] (explicit migration; still unimplemented).
    pub fn ensure_schema(&self, version: u32, document: &str) -> Result<()> {
        schema::validate_document(document)?;
        let mut state = self.read_state()?;
        match &state.schema {
            Some(schema) if schema.version != version => {
                return Err(Error::SchemaConflict {
                    expected: version,
                    found: schema.version,
                });
            }
            Some(_) => {
                // Legacy files may lack a durable catalog blob — hydrate once.
                if state.schema_catalog.is_none() {
                    let stored = state
                        .schema
                        .as_ref()
                        .map(|s| s.document.clone())
                        .unwrap_or_default();
                    if let Some(catalog) = ddl::catalog_for_schema(&stored, None)? {
                        state.schema_catalog = Some(catalog);
                        return self.write_state(&state);
                    }
                }
                return Ok(());
            }
            None => {
                let catalog = ddl::catalog_for_schema(document, None)?;
                state.schema = Some(SchemaVersion {
                    version,
                    document: document.to_owned(),
                });
                state.schema_catalog = catalog;
            }
        }
        self.write_state(&state)
    }

    /// Explicit schema migration between versions.
    ///
    /// Intentionally separate from [`Self::ensure_schema`]. Not implemented yet.
    pub fn migrate_schema(&self, from_version: u32, to_version: u32, document: &str) -> Result<()> {
        let _ = (from_version, to_version, document);
        Err(Error::Unsupported(
            "schema migration (use ensure_schema only for the initial schema)",
        ))
    }

    /// Begin a connection-local write transaction.
    ///
    /// While open, [`Self::put`] / [`Self::put_row`] / schema writes update a
    /// draft only. [`Self::commit`] persists one atomic snapshot;
    /// [`Self::rollback`] discards it. Nested transactions are not supported.
    ///
    /// Acquires the process-local exclusive write lease until commit/rollback.
    pub fn begin(&self) -> Result<()> {
        if self.ddl_draft_is_open() {
            return Err(Error::Transaction {
                message: "cannot begin data transaction while a DDL session is open".into(),
            });
        }
        {
            let guard = self
                .txn
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if guard.is_some() {
                return Err(Error::Transaction {
                    message: "transaction already open".into(),
                });
            }
        }
        self.acquire_long_write_lease()?;
        let ddl_rev = match self.ddl_revision() {
            Ok(v) => v,
            Err(err) => {
                self.release_long_write_lease();
                return Err(err);
            }
        };
        let state = match self.read_committed_state() {
            Ok(s) => s,
            Err(err) => {
                self.release_long_write_lease();
                return Err(err);
            }
        };
        *self
            .txn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(state);
        *self
            .txn_ddl_revision
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(ddl_rev);
        Ok(())
    }

    /// Persist the open transaction draft as one durable snapshot.
    pub fn commit(&self) -> Result<()> {
        let draft = {
            let mut guard = self
                .txn
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            guard.take().ok_or_else(|| Error::Transaction {
                message: "no transaction in progress".into(),
            })?
        };
        let txn_rev = self
            .txn_ddl_revision
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        if let Some(txn_rev) = txn_rev {
            let current = self.ddl_revision()?;
            if txn_rev != current {
                // Restore draft so the caller can still rollback explicitly.
                *self
                    .txn
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(draft);
                *self
                    .txn_ddl_revision
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(txn_rev);
                return Err(session::transaction_stale_ddl(txn_rev, current));
            }
        }
        match self.write_committed_state(&draft) {
            Ok(()) => {
                self.release_long_write_lease();
                Ok(())
            }
            Err(err) => {
                // Restore draft + keep lease so the caller can retry or rollback.
                *self
                    .txn
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(draft);
                if let Some(txn_rev) = txn_rev {
                    *self
                        .txn_ddl_revision
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(txn_rev);
                }
                Err(err)
            }
        }
    }

    /// Discard the open transaction draft.
    pub fn rollback(&self) -> Result<()> {
        let mut guard = self
            .txn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if guard.take().is_none() {
            return Err(Error::Transaction {
                message: "no transaction in progress".into(),
            });
        }
        *self
            .txn_ddl_revision
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
        self.release_long_write_lease();
        Ok(())
    }

    /// True while a transaction started by [`Self::begin`] is open.
    pub fn in_transaction(&self) -> bool {
        self.txn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_some()
    }

    /// Insert or replace a typed table row validated against the VOS schema.
    pub fn put_row(&self, table: &str, row: Row) -> Result<()> {
        let document = self.parsed_schema()?.ok_or_else(|| Error::Schema {
            message: "database has no VOS schema".into(),
            span: None,
            hint: Some("call ensure_schema with a non-empty VOS document first".into()),
        })?;
        let table_def = table::find_table(&document, table)?;
        let pk = table::validate_row(table_def, &row)?;
        let key = table::row_key(table, &pk)?;
        let pk_key_bytes = key.as_bytes();
        let mut state = self.read_state()?;
        for field in table_def.fields.iter().filter(|f| f.is_unique()) {
            let Some(value) = row.get(&field.name) else {
                continue;
            };
            if matches!(value, Value::Null) {
                continue;
            }
            if let Some(existing_pk) =
                index::lookup_unique(&state.records, table, &field.name, value)?
            {
                if existing_pk != pk_key_bytes {
                    return Err(Error::Schema {
                        message: format!(
                            "unique constraint failed on `{}.{}`",
                            table_def.name, field.name
                        ),
                        span: Some((field.span.start, field.span.end)),
                        hint: Some("choose a different value for the unique field".into()),
                    });
                }
            }
        }
        index::clear_unique_indexes_for_pk(&mut state.records, table, table_def, &pk)?;
        let payload = table::encode_row(&row)?;
        state.records.insert(key, payload);
        index::write_unique_indexes(&mut state.records, table, table_def, &pk, &row)?;
        self.write_state(&state)
    }

    /// Fetch a typed table row by primary key.
    pub fn get_row(&self, table: &str, primary_key: &Value) -> Result<Option<Row>> {
        let key = table::row_key(table, primary_key)?;
        match self.get(&key)? {
            Some(bytes) => Ok(Some(table::decode_row(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Delete a typed table row by primary key.
    pub fn delete_row(&self, table: &str, primary_key: &Value) -> Result<bool> {
        let document = self.parsed_schema()?.ok_or_else(|| Error::Schema {
            message: "database has no VOS schema".into(),
            span: None,
            hint: None,
        })?;
        let table_def = table::find_table(&document, table)?;
        let key = table::row_key(table, primary_key)?;
        let mut state = self.read_state()?;
        let Some(bytes) = state.records.remove(&key) else {
            return Ok(false);
        };
        let row = table::decode_row(&bytes)?;
        index::clear_unique_indexes_for_pk(&mut state.records, table, table_def, primary_key)?;
        // Also clear by decoded row values in case indexes drifted.
        for field in table_def.fields.iter().filter(|f| f.is_unique()) {
            if let Some(value) = row.get(&field.name) {
                if !matches!(value, Value::Null) {
                    let idx = index::unique_index_key(table, &field.name, value)?;
                    state.records.remove(&idx);
                }
            }
        }
        self.write_state(&state)?;
        Ok(true)
    }

    /// Fetch a typed table row by a unique field value (`@field` / `[unique]`).
    pub fn get_row_by_unique(
        &self,
        table: &str,
        field: &str,
        value: &Value,
    ) -> Result<Option<Row>> {
        let state = self.read_state()?;
        let Some(pk_key) = index::lookup_unique(&state.records, table, field, value)? else {
            return Ok(None);
        };
        let key = String::from_utf8(pk_key).map_err(|_| Error::Corrupt("index pk is not utf-8"))?;
        match state.records.get(&key) {
            Some(bytes) => Ok(Some(table::decode_row(bytes)?)),
            None => Ok(None),
        }
    }

    /// Parse and execute a VOS operation-language program.
    ///
    /// Supports a v1 subset: `Type.get`, `Type {…}.insert()`, filter / map /
    /// sort / skip / take / collect, and entity/collection update/delete.
    pub fn query(&self, vos_program: &str) -> Result<Vec<table::Row>> {
        query::execute_program(self, vos_program)
    }

    /// List all stored rows for `table` (full scan; interim until indexes land).
    pub fn scan_table_rows(&self, table: &str) -> Result<Vec<(Value, Row)>> {
        let document = self.parsed_schema()?.ok_or_else(|| Error::Schema {
            message: "database has no VOS schema".into(),
            span: None,
            hint: None,
        })?;
        let table_def = table::find_table(&document, table)?;
        let pk_name = table_def
            .primary_fields()
            .next()
            .map(|f| f.name.as_str())
            .ok_or_else(|| Error::Schema {
                message: format!("table `{table}` has no primary key"),
                span: None,
                hint: None,
            })?;
        let prefix = table::table_row_prefix(table);
        let state = self.read_state()?;
        let mut out = Vec::new();
        for (key, bytes) in state.records.range(prefix.clone()..) {
            if !key.starts_with(&prefix) {
                break;
            }
            let row = table::decode_row(bytes)?;
            let pk = row
                .get(pk_name)
                .cloned()
                .ok_or_else(|| Error::Corrupt("stored row missing primary key field"))?;
            out.push((pk, row));
        }
        Ok(out)
    }

    /// Insert or replace a raw byte record keyed by `key`.
    ///
    /// Inline row storage is **allowed** but **not recommended** for large
    /// payloads — prefer [`Self::put_chunk`] / [`Self::put_file_chunked`] into
    /// the unified `objects/` CAS (see `INLINE_BYTES_MAX`).
    pub fn put(&self, key: impl Into<String>, value: impl AsRef<[u8]>) -> Result<()> {
        let mut state = self.read_state()?;
        state.records.insert(key.into(), value.as_ref().to_vec());
        self.write_state(&state)
    }

    /// Fetch a raw byte record by key.
    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.read_state()?.records.get(key).cloned())
    }

    /// Store one CAS object at `objects/hash-2/<hash>.bytes`.
    pub fn put_chunk(&self, kind: ObjectKind, bytes: &[u8]) -> Result<ObjectRef> {
        self.objects.put_chunk(kind, bytes)
    }

    /// Store a vector payload in CAS (`ObjectKind::VectorPayload`).
    pub fn put_vector(&self, vector: &Vector) -> Result<ObjectRef> {
        self.objects.put_vector(vector)
    }

    /// Load a vector payload from CAS.
    pub fn get_vector(&self, object: &ObjectRef) -> Result<Vector> {
        self.objects.get_vector(object)
    }

    /// Read CAS bytes (fault-in to hot tier when needed).
    pub fn get_object(&self, object: &ObjectRef) -> Result<Arc<[u8]>> {
        self.objects.get_object(object)
    }

    /// Chunk a logical file into CAS objects.
    pub fn put_file_chunked(
        &self,
        reader: impl std::io::Read,
        chunk_size: usize,
    ) -> Result<ChunkManifest> {
        self.objects.put_file_chunked(reader, chunk_size)
    }

    /// Range-read a chunked file without assembling the whole payload.
    pub fn read_file_range(
        &self,
        manifest: &ChunkManifest,
        offset: u64,
        len: usize,
    ) -> Result<Vec<u8>> {
        self.objects.read_file_range(manifest, offset, len)
    }

    /// Pin an object in the hot tier.
    pub fn pin_object(&self, object: &ObjectRef) -> Result<Tier> {
        self.objects.pin_object(object)
    }

    /// Evict an object from the hot tier (disk CAS remains).
    pub fn evict_object(&self, object: &ObjectRef) -> Result<Tier> {
        self.objects.evict_object(object)
    }

    /// Current hot/cold hint for an object.
    pub fn object_tier(&self, object: &ObjectRef) -> Tier {
        self.objects.tier_of(object)
    }

    /// Register a Rust scalar UDF with a fixed arity.
    ///
    /// Replaces any previous registration with the same `name`. UDFs live only
    /// in this process; reopen the file and they are gone until re-registered.
    pub fn create_scalar<F>(&self, name: &str, n_args: usize, func: F) -> Result<()>
    where
        F: Fn(&[Value]) -> Result<Value> + Send + Sync + 'static,
    {
        self.create_scalar_variadic(name, Some(n_args), func)
    }

    /// Register a Rust scalar UDF; `None` arity accepts any argument count.
    pub fn create_scalar_variadic<F>(&self, name: &str, arity: Option<usize>, func: F) -> Result<()>
    where
        F: Fn(&[Value]) -> Result<Value> + Send + Sync + 'static,
    {
        let boxed: Arc<ScalarFn> = Arc::new(func);
        let udf: Arc<dyn ScalarUdf> = Arc::new(ClosureUdf::new(arity, boxed));
        self.register_scalar(name, udf)
    }

    /// Register an object-safe [`ScalarUdf`].
    pub fn register_scalar(&self, name: &str, udf: Arc<dyn ScalarUdf>) -> Result<()> {
        if name.is_empty() {
            return Err(Error::Udf {
                name: String::new(),
                message: "UDF name must not be empty".into(),
            });
        }
        self.udfs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(name.to_owned(), RegisteredUdf { udf });
        Ok(())
    }

    /// Remove a previously registered scalar UDF.
    pub fn remove_scalar(&self, name: &str) -> Result<()> {
        let removed = self
            .udfs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(name)
            .is_some();
        if removed {
            Ok(())
        } else {
            Err(Error::UdfNotFound {
                name: name.to_owned(),
            })
        }
    }

    /// Names of scalar UDFs registered on this connection.
    pub fn list_scalars(&self) -> Vec<String> {
        self.udfs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .keys()
            .cloned()
            .collect()
    }

    /// Invoke a registered scalar UDF (also used by future query execution).
    pub fn call_scalar(&self, name: &str, args: &[Value]) -> Result<Value> {
        let guard = self
            .udfs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let entry = guard.get(name).ok_or_else(|| Error::UdfNotFound {
            name: name.to_owned(),
        })?;
        if let Some(expected) = entry.udf.arity() {
            if args.len() != expected {
                return Err(Error::UdfArity {
                    name: name.to_owned(),
                    expected,
                    got: args.len(),
                });
            }
        }
        entry.udf.call(args).map_err(|error| match error {
            Error::Udf { .. }
            | Error::UdfArity { .. }
            | Error::UdfNotFound { .. }
            | Error::ObjectNotFound { .. }
            | Error::ObjectCorrupt { .. }
            | Error::Unsupported(_)
            | Error::Transaction { .. } => error,
            other => Error::Udf {
                name: name.to_owned(),
                message: other.to_string(),
            },
        })
    }

    fn read_state(&self) -> Result<State> {
        {
            let guard = self
                .txn
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(draft) = guard.as_ref() {
                return Ok(draft.clone());
            }
        }
        self.read_committed_state()
    }

    fn write_state(&self, state: &State) -> Result<()> {
        {
            let mut guard = self
                .txn
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(draft) = guard.as_mut() {
                *draft = state.clone();
                return Ok(());
            }
        }
        self.write_committed_state(state)
    }

    fn read_committed_state(&self) -> Result<State> {
        let _read_ticket = match &self.write_gate {
            Some(gate) => Some(gate.try_enter_read(self.conn_id)?),
            None => None,
        };
        match &self.backend {
            Backend::File { path, layout, .. } => match layout {
                FileLayout::Snapshot => load_snapshot_state(path),
                FileLayout::Paged(pager) => {
                    let mut guard = pager
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    load_paged_committed(path, &mut guard)
                }
            },
            Backend::Memory { state } => Ok(state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone()),
        }
    }

    fn write_committed_state(&self, state: &State) -> Result<()> {
        self.with_write_lease(|| self.write_committed_state_unlocked(state))
    }

    fn write_committed_state_unlocked(&self, state: &State) -> Result<()> {
        match &self.backend {
            Backend::File {
                path,
                journal_mode,
                layout,
            } => {
                let mode = *journal_mode
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                match layout {
                    FileLayout::Snapshot => match mode {
                        JournalMode::Delete => write_main(path, state),
                        JournalMode::Wal => {
                            let payload = encode(state)?;
                            append_snapshot_frame(path, &payload)
                        }
                    },
                    FileLayout::Paged(pager) => match mode {
                        JournalMode::Delete => {
                            let mut guard = pager
                                .lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner());
                            write_paged_state(&mut guard, state)
                        }
                        JournalMode::Wal => {
                            // Same type-1 full-state frames as snapshot WAL; main
                            // pages stay unchanged until checkpoint.
                            let payload = encode(state)?;
                            append_snapshot_frame(path, &payload)
                        }
                    },
                }
            }
            Backend::Memory { state: slot } => {
                *slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = state.clone();
                Ok(())
            }
        }
    }

    fn holds_long_write_lease(&self) -> bool {
        *self
            .write_lease
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn acquire_long_write_lease(&self) -> Result<()> {
        let Some(gate) = &self.write_gate else {
            return Ok(());
        };
        if self.holds_long_write_lease() {
            return Ok(());
        }
        gate.try_acquire(self.conn_id)?;
        *self
            .write_lease
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = true;
        Ok(())
    }

    fn release_long_write_lease(&self) {
        let Some(gate) = &self.write_gate else {
            return;
        };
        let mut leased = self
            .write_lease
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !*leased {
            return;
        }
        *leased = false;
        drop(leased);
        gate.release(self.conn_id);
    }

    /// Run `f` under the write lease (reuses a long lease when already held).
    fn with_write_lease<T>(&self, f: impl FnOnce() -> Result<T>) -> Result<T> {
        let Some(gate) = &self.write_gate else {
            return f();
        };
        if self.holds_long_write_lease() {
            return f();
        }
        gate.try_acquire(self.conn_id)?;
        let result = f();
        gate.release(self.conn_id);
        result
    }

    fn ddl_draft_is_open(&self) -> bool {
        self.ddl_draft
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_some()
    }

    pub(crate) fn open_ddl_draft(&self) -> Result<()> {
        {
            let txn = self
                .txn
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if txn.is_some() {
                return Err(Error::Transaction {
                    message: "cannot open DDL session while a data transaction is open".into(),
                });
            }
        }
        {
            let guard = self
                .ddl_draft
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if guard.is_some() {
                return Err(Error::Transaction {
                    message: "DDL session already open".into(),
                });
            }
        }
        self.acquire_long_write_lease()?;
        let opened = (|| {
            let state = self.read_committed_state()?;
            let Some(schema) = state.schema.as_ref() else {
                return Err(Error::Schema {
                    message: "VOS-DDL-SESSION-REQUIRED: database has no schema".into(),
                    span: None,
                    hint: Some("call ensure_schema before begin_ddl_session".into()),
                });
            };
            let catalog = ddl::catalog_for_schema(&schema.document, state.schema_catalog.as_ref())?
                .ok_or_else(|| Error::Schema {
                    message: "VOS-DDL-SESSION-REQUIRED: empty schema cannot be mutated".into(),
                    span: None,
                    hint: None,
                })?;
            *self
                .ddl_draft
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(catalog);
            Ok(())
        })();
        if opened.is_err() {
            self.release_long_write_lease();
        }
        opened
    }

    pub(crate) fn ddl_draft_catalog(&self) -> Result<schema_catalog::SchemaCatalog> {
        self.ddl_draft
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
            .ok_or_else(ddl::require_ddl_session)
    }

    pub(crate) fn ddl_mutate<T>(
        &self,
        f: impl FnOnce(&mut schema_catalog::SchemaCatalog) -> Result<T>,
    ) -> Result<T> {
        let mut guard = self
            .ddl_draft
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let draft = guard.as_mut().ok_or_else(ddl::require_ddl_session)?;
        f(draft)
    }

    pub(crate) fn commit_ddl_draft(&self) -> Result<()> {
        let draft = {
            let mut guard = self
                .ddl_draft
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            guard.take().ok_or_else(ddl::require_ddl_session)?
        };
        match (|| {
            let mut state = self.read_committed_state()?;
            let Some(schema) = state.schema.as_mut() else {
                return Err(Error::Schema {
                    message: "missing schema while committing DDL".into(),
                    span: None,
                    hint: None,
                });
            };
            let document = draft.to_canonical_source();
            schema::validate_document(&document)?;
            schema.document = document;
            state.schema_catalog = Some(draft.clone());
            self.write_committed_state(&state)
        })() {
            Ok(()) => {
                self.release_long_write_lease();
                Ok(())
            }
            Err(err) => {
                let mut guard = self
                    .ddl_draft
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *guard = Some(draft);
                Err(err)
            }
        }
    }

    pub(crate) fn rollback_ddl_draft(&self) {
        let mut guard = self
            .ddl_draft
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let had = guard.take().is_some();
        drop(guard);
        if had {
            self.release_long_write_lease();
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.release_long_write_lease();
    }
}

/// Library version string (Cargo package version).
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn open_file_layout(path: &Path) -> Result<FileLayout> {
    use std::io::Read;
    let mut file = fs::File::open(path).map_err(Error::Io)?;
    let mut head = [0u8; 5];
    let n = file.read(&mut head).map_err(Error::Io)?;
    if n < 5 {
        return open_short_or_torn_paged(path, "file too short");
    }
    if &head == PAGER_MAGIC {
        match Pager::open(path) {
            Ok(pager) => Ok(FileLayout::Paged(Mutex::new(pager))),
            Err(Error::Corrupt(reason)) => open_short_or_torn_paged(path, reason),
            Err(err) => Err(err),
        }
    } else if &head == MAGIC {
        Ok(FileLayout::Snapshot)
    } else {
        // Garbage header: still try WAL-only paged repair when frames exist.
        open_short_or_torn_paged(path, "unknown file header")
    }
}

/// When the paged main file is truncated / checksum-corrupt / unreadable, rebuild
/// it from the last complete type-1 WAL frame when one exists.
fn open_short_or_torn_paged(path: &Path, reason: &'static str) -> Result<FileLayout> {
    let Some(payload) = last_complete_wal_payload(path)? else {
        return Err(Error::Corrupt(reason));
    };
    let state = decode(&payload)?;
    let _ = fs::remove_file(path);
    let mut pager = Pager::create(path)?;
    catalog_page::init_empty(&mut pager)?;
    write_paged_state(&mut pager, &state)?;
    Ok(FileLayout::Paged(Mutex::new(pager)))
}

fn prepare_journal_sidecars(path: &Path, mode: JournalMode, layout: &FileLayout) -> Result<()> {
    match mode {
        JournalMode::Wal => ensure_wal_sidecars(path),
        JournalMode::Delete => {
            if !(wal_path(path).exists() || shm_path(path).exists()) {
                return Ok(());
            }
            match layout {
                FileLayout::Snapshot => {
                    let state = load_snapshot_state(path)?;
                    write_main(path, &state)?;
                }
                FileLayout::Paged(pager) => {
                    let mut guard = pager
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    let state = load_paged_committed(path, &mut guard)?;
                    write_paged_state(&mut guard, &state)?;
                }
            }
            remove_wal_sidecars(path)?;
            Ok(())
        }
    }
}

fn load_snapshot_state(path: &Path) -> Result<State> {
    let main = fs::read(path)?;
    let merged = replay_wal_snapshots(path, main)?;
    decode(&merged)
}

fn load_paged_state(pager: &mut Pager) -> Result<State> {
    // Refresh meta `page_count` so peer-committed page extensions are visible.
    let _ = pager.read_page(0)?;
    let root = pager::read_catalog_root(pager)?;
    let catalog = catalog_page::read_catalog(pager, root)?;
    let records = paged_store::load_records(pager)?;
    Ok(State {
        schema: catalog.schema,
        schema_catalog: catalog.schema_catalog,
        records,
    })
}

/// Load paged main pages, then apply any type-1 WAL frames (last complete wins).
///
/// If the main page image is truncated or checksum-corrupt mid-write but the WAL
/// still holds complete type-1 frames, rebuild main from the last WAL payload
/// (type-1 frames are full-state snapshots, so main is not required for truth).
fn load_paged_committed(path: &Path, pager: &mut Pager) -> Result<State> {
    match load_paged_state(pager) {
        Ok(base) => {
            if !wal_path(path).exists() {
                return Ok(base);
            }
            let encoded = encode(&base)?;
            let merged = replay_wal_snapshots(path, encoded)?;
            decode(&merged)
        }
        Err(Error::Corrupt(_)) => repair_paged_main_from_wal(path, pager),
        Err(err) => Err(err),
    }
}

/// Rewrite a torn/corrupt paged main from the last complete WAL snapshot.
fn repair_paged_main_from_wal(path: &Path, pager: &mut Pager) -> Result<State> {
    let Some(payload) = last_complete_wal_payload(path)? else {
        return Err(Error::Corrupt(
            "paged main corrupt and wal has no complete frames",
        ));
    };
    let state = decode(&payload)?;
    pager.reformat()?;
    catalog_page::init_empty(pager)?;
    write_paged_state(pager, &state)?;
    Ok(state)
}

fn write_paged_state(pager: &mut Pager, state: &State) -> Result<()> {
    let catalog =
        catalog_page::CatalogState::new(state.schema.clone(), state.schema_catalog.clone());
    catalog_page::write_catalog(pager, &catalog)?;
    paged_store::store_records(pager, &state.records)?;
    Ok(())
}

fn write_main(path: &Path, state: &State) -> Result<()> {
    fs::write(path, encode(state)?).map_err(Error::Io)
}

fn encode(state: &State) -> Result<Vec<u8>> {
    let mut bytes = MAGIC.to_vec();
    match &state.schema {
        Some(schema) => match &state.schema_catalog {
            Some(catalog) => {
                bytes.push(2);
                bytes.extend(schema.version.to_le_bytes());
                write_bytes(&mut bytes, schema.document.as_bytes())?;
                write_bytes(&mut bytes, &catalog.encode()?)?;
            }
            None => {
                bytes.push(1);
                bytes.extend(schema.version.to_le_bytes());
                write_bytes(&mut bytes, schema.document.as_bytes())?;
            }
        },
        None => bytes.push(0),
    }
    write_u32(&mut bytes, state.records.len())?;
    for (key, value) in &state.records {
        write_bytes(&mut bytes, key.as_bytes())?;
        write_bytes(&mut bytes, value)?;
    }
    Ok(bytes)
}

fn decode(bytes: &[u8]) -> Result<State> {
    let mut cursor = 0;
    if take(bytes, &mut cursor, MAGIC.len())? != MAGIC {
        return Err(Error::Corrupt("unknown file header"));
    }
    let (schema, schema_catalog) = match take(bytes, &mut cursor, 1)? {
        [0] => (None, None),
        [1] => {
            let schema = SchemaVersion {
                version: read_u32(bytes, &mut cursor)?,
                document: String::from_utf8(read_bytes(bytes, &mut cursor)?)
                    .map_err(|_| Error::Corrupt("schema is not UTF-8"))?,
            };
            // Rebuild identity catalog from document when the file predates marker 2.
            let catalog = if schema::is_empty_document(&schema.document) {
                None
            } else {
                let doc = schema::parse_document(&schema.document)?;
                Some(schema_catalog::SchemaCatalog::from_document(&doc)?)
            };
            (Some(schema), catalog)
        }
        [2] => {
            let schema = SchemaVersion {
                version: read_u32(bytes, &mut cursor)?,
                document: String::from_utf8(read_bytes(bytes, &mut cursor)?)
                    .map_err(|_| Error::Corrupt("schema is not UTF-8"))?,
            };
            let blob = read_bytes(bytes, &mut cursor)?;
            let catalog = schema_catalog::SchemaCatalog::decode(&blob)?;
            (Some(schema), Some(catalog))
        }
        _ => return Err(Error::Corrupt("unknown schema marker")),
    };
    let count = read_u32(bytes, &mut cursor)?;
    let mut records = BTreeMap::new();
    for _ in 0..count {
        let key = String::from_utf8(read_bytes(bytes, &mut cursor)?)
            .map_err(|_| Error::Corrupt("record key is not UTF-8"))?;
        records.insert(key, read_bytes(bytes, &mut cursor)?);
    }
    if cursor != bytes.len() {
        return Err(Error::Corrupt("trailing data"));
    }
    Ok(State {
        schema,
        schema_catalog,
        records,
    })
}

fn write_u32(bytes: &mut Vec<u8>, value: usize) -> Result<()> {
    let value = u32::try_from(value).map_err(|_| Error::Corrupt("value exceeds 4 GiB"))?;
    bytes.extend(value.to_le_bytes());
    Ok(())
}

fn write_bytes(bytes: &mut Vec<u8>, value: &[u8]) -> Result<()> {
    write_u32(bytes, value.len())?;
    bytes.extend(value);
    Ok(())
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32> {
    let raw = take(bytes, cursor, 4)?;
    Ok(u32::from_le_bytes(
        raw.try_into().expect("requested exactly 4 bytes"),
    ))
}

fn read_bytes(bytes: &[u8], cursor: &mut usize) -> Result<Vec<u8>> {
    let length = usize::try_from(read_u32(bytes, cursor)?).expect("u32 fits usize");
    Ok(take(bytes, cursor, length)?.to_vec())
}

fn take<'a>(bytes: &'a [u8], cursor: &mut usize, length: usize) -> Result<&'a [u8]> {
    let end = cursor
        .checked_add(length)
        .ok_or(Error::Corrupt("length overflow"))?;
    let value = bytes
        .get(*cursor..end)
        .ok_or(Error::Corrupt("unexpected EOF"))?;
    *cursor = end;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("yydb-{label}-{nonce}.yydb"))
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(wal_path(path));
        let _ = fs::remove_file(shm_path(path));
        let _ = fs::remove_file(concurrency::write_lock_path(path));
        let mut objects = path.as_os_str().to_owned();
        objects.push(".objects");
        let _ = fs::remove_dir_all(PathBuf::from(objects));
    }

    #[test]
    fn stores_schema_and_records_in_one_reopenable_file() {
        let path = temp_db("reopen");
        let conn = Connection::open(&path).unwrap();
        conn.ensure_schema(1, "table Project { @@id: uuid, title: utf8 }")
            .unwrap();
        conn.put("project/meta", b"Spark").unwrap();
        drop(conn);

        let reopened = Connection::open(&path).unwrap();
        assert_eq!(reopened.schema().unwrap().unwrap().version, 1);
        assert_eq!(reopened.record_count().unwrap(), 1);
        assert_eq!(
            reopened.get("project/meta").unwrap(),
            Some(b"Spark".to_vec())
        );
        cleanup(&path);
    }

    #[test]
    fn create_paged_and_persist_schema_on_catalog_pages() {
        let path = temp_db("paged");
        let conn = Connection::create_paged(&path).unwrap();
        assert!(conn.is_paged());
        conn.ensure_schema(1, "table Project { @@id: uuid, title: utf8 }")
            .unwrap();
        drop(conn);

        let reopened = Connection::open(&path).unwrap();
        assert!(reopened.is_paged());
        assert_eq!(reopened.schema().unwrap().unwrap().version, 1);
        assert!(reopened.field_catalog().unwrap().is_some());
        cleanup(&path);
    }

    #[test]
    fn create_paged_persists_kv_records_on_btree() {
        let path = temp_db("paged-kv");
        let conn = Connection::create_paged(&path).unwrap();
        conn.ensure_schema(1, "table Project { @@id: uuid, title: utf8 }")
            .unwrap();
        conn.put("project/meta", b"Spark").unwrap();
        conn.put("other", b"data").unwrap();
        assert_eq!(conn.record_count().unwrap(), 2);
        drop(conn);

        let reopened = Connection::open(&path).unwrap();
        assert!(reopened.is_paged());
        assert_eq!(reopened.record_count().unwrap(), 2);
        assert_eq!(
            reopened.get("project/meta").unwrap(),
            Some(b"Spark".to_vec())
        );
        assert_eq!(reopened.get("other").unwrap(), Some(b"data".to_vec()));
        cleanup(&path);
    }

    #[test]
    fn create_paged_typed_rows_reopen_from_btree() {
        let path = temp_db("paged-rows");
        cleanup(&path);
        let conn = Connection::create_paged(&path).unwrap();
        conn.ensure_schema(
            1,
            r#"
            table Note {
                @@id: uuid,
                title: utf8,
            }
            "#,
        )
        .unwrap();
        let id = uuid::Uuid::parse_str("00000000-0000-0000-0000-0000000000a1").unwrap();
        let mut row = Row::new();
        row.insert("id", Value::Uuid(id));
        row.insert("title", Value::Text("alpha".into()));
        conn.put_row("Note", row.clone()).unwrap();
        drop(conn);

        let reopened = Connection::open(&path).unwrap();
        assert!(reopened.is_paged());
        let got = reopened.get_row("Note", &Value::Uuid(id)).unwrap().unwrap();
        assert_eq!(got.get("title"), Some(&Value::Text("alpha".into())));
        cleanup(&path);
    }

    #[test]
    fn create_paged_unique_index_roundtrip_duplicate_and_delete() {
        let path = temp_db("paged-unique");
        cleanup(&path);
        let conn = Connection::create_paged(&path).unwrap();
        conn.ensure_schema(
            1,
            r#"
            table User {
                @@user_id: uuid,
                @user_name: utf8,
            }
            "#,
        )
        .unwrap();
        let id = uuid::Uuid::parse_str("00000000-0000-0000-0000-0000000000dd").unwrap();
        let mut row = Row::new();
        row.insert("user_id", Value::Uuid(id));
        row.insert("user_name", Value::Text("dana".into()));
        conn.put_row("User", row.clone()).unwrap();

        let other = uuid::Uuid::parse_str("00000000-0000-0000-0000-0000000000de").unwrap();
        let mut dup = Row::new();
        dup.insert("user_id", Value::Uuid(other));
        dup.insert("user_name", Value::Text("dana".into()));
        assert!(matches!(
            conn.put_row("User", dup),
            Err(Error::Schema { message, .. }) if message.contains("unique")
        ));
        drop(conn);

        let reopened = Connection::open(&path).unwrap();
        assert!(reopened.is_paged());
        let fetched = reopened
            .get_row_by_unique("User", "user_name", &Value::Text("dana".into()))
            .unwrap()
            .unwrap();
        assert_eq!(fetched, row);
        assert!(reopened.delete_row("User", &Value::Uuid(id)).unwrap());
        assert_eq!(
            reopened
                .get_row_by_unique("User", "user_name", &Value::Text("dana".into()))
                .unwrap(),
            None
        );
        // Index entry must stay cleared across another reopen.
        drop(reopened);
        let again = Connection::open(&path).unwrap();
        assert_eq!(
            again
                .get_row_by_unique("User", "user_name", &Value::Text("dana".into()))
                .unwrap(),
            None
        );
        cleanup(&path);
    }

    #[test]
    fn open_creates_paged_by_default() {
        let path = temp_db("default-paged");
        cleanup(&path);
        let conn = Connection::open(&path).unwrap();
        assert!(conn.is_paged());
        drop(conn);
        let reopened = Connection::open(&path).unwrap();
        assert!(reopened.is_paged());
        cleanup(&path);
    }

    #[test]
    fn open_with_flags_create_paged_layout() {
        let path = temp_db("flags-paged");
        cleanup(&path);
        let conn = Connection::open_with_flags(&path, OpenFlags::create_paged()).unwrap();
        assert!(conn.is_paged());
        conn.put("k", b"v").unwrap();
        drop(conn);
        let reopened = Connection::open(&path).unwrap();
        assert!(reopened.is_paged());
        assert_eq!(reopened.get("k").unwrap(), Some(b"v".to_vec()));
        cleanup(&path);
    }

    #[test]
    fn snapshot_files_still_open_as_snapshot_era() {
        let path = temp_db("snap");
        cleanup(&path);
        let conn = Connection::open_with_flags(&path, OpenFlags::create_snapshot()).unwrap();
        assert!(!conn.is_paged());
        conn.put("k", b"v").unwrap();
        drop(conn);

        let reopened = Connection::open(&path).unwrap();
        assert!(!reopened.is_paged());
        assert_eq!(reopened.get("k").unwrap(), Some(b"v".to_vec()));
        cleanup(&path);
    }

    #[test]
    fn open_in_memory_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(conn.path().is_none());
        conn.ensure_schema(2, "table Demo { @@id: uuid }").unwrap();
        conn.put("k", b"v").unwrap();
        assert_eq!(conn.get("k").unwrap(), Some(b"v".to_vec()));
        assert_eq!(conn.schema().unwrap().unwrap().version, 2);
        assert_eq!(conn.record_count().unwrap(), 1);
        let parsed = conn.parsed_schema().unwrap().unwrap();
        assert_eq!(parsed.tables().next().unwrap().name, "Demo");
        assert!(matches!(
            conn.migrate_schema(2, 3, "table Demo { @@id: uuid, name: utf8 }"),
            Err(Error::Unsupported(_))
        ));
    }

    #[test]
    fn typed_table_row_roundtrip_and_validation() {
        let conn = Connection::open_in_memory().unwrap();
        conn.ensure_schema(
            1,
            r#"
            table Setting {
                @@id: uuid,
                key: utf8,
                value: utf8,
            }
            "#,
        )
        .unwrap();
        let id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let mut row = Row::new();
        row.insert("id", Value::Uuid(id));
        row.insert("key", Value::Text("theme".into()));
        row.insert("value", Value::Text("dark".into()));
        conn.put_row("Setting", row.clone()).unwrap();
        assert_eq!(
            conn.get_row("Setting", &Value::Uuid(id)).unwrap(),
            Some(row)
        );

        let mut bad = Row::new();
        bad.insert("id", Value::Uuid(id));
        bad.insert("key", Value::I64(1));
        assert!(matches!(
            conn.put_row("Setting", bad),
            Err(Error::Schema { .. })
        ));
        assert!(conn.delete_row("Setting", &Value::Uuid(id)).unwrap());
        assert_eq!(conn.get_row("Setting", &Value::Uuid(id)).unwrap(), None);
    }

    #[test]
    fn file_backed_typed_rows_reopen_and_scan() {
        let path = temp_db("typed-rows");
        cleanup(&path);
        let conn = Connection::open(&path).unwrap();
        conn.ensure_schema(
            1,
            r#"
            table Note {
                @@id: uuid,
                title: utf8,
            }
            "#,
        )
        .unwrap();
        let a = uuid::Uuid::parse_str("00000000-0000-0000-0000-0000000000a1").unwrap();
        let b = uuid::Uuid::parse_str("00000000-0000-0000-0000-0000000000b2").unwrap();
        for (id, title) in [(a, "alpha"), (b, "beta")] {
            let mut row = Row::new();
            row.insert("id", Value::Uuid(id));
            row.insert("title", Value::Text(title.into()));
            conn.put_row("Note", row).unwrap();
        }
        drop(conn);

        let reopened = Connection::open(&path).unwrap();
        let got = reopened.get_row("Note", &Value::Uuid(a)).unwrap().unwrap();
        assert_eq!(got.get("title"), Some(&Value::Text("alpha".into())));
        let scanned = reopened.scan_table_rows("Note").unwrap();
        assert_eq!(scanned.len(), 2);
        let titles: Vec<_> = scanned
            .iter()
            .map(|(_, r)| r.get("title").cloned())
            .collect();
        assert!(titles.contains(&Some(Value::Text("alpha".into()))));
        assert!(titles.contains(&Some(Value::Text("beta".into()))));
        let via_query = reopened
            .query(r#"Note.filter(x => x.title == "beta").collect()"#)
            .unwrap();
        assert_eq!(via_query.len(), 1);
        assert_eq!(via_query[0].get("title"), Some(&Value::Text("beta".into())));
        drop(reopened);
        cleanup(&path);
    }

    #[test]
    fn file_backed_unique_index_reopens_from_primary_file() {
        let path = temp_db("unique-index");
        cleanup(&path);
        let conn = Connection::open(&path).unwrap();
        conn.ensure_schema(
            1,
            r#"
            table User {
                @@user_id: uuid,
                @user_name: utf8,
            }
            "#,
        )
        .unwrap();
        let id = uuid::Uuid::parse_str("00000000-0000-0000-0000-0000000000cc").unwrap();
        let mut row = Row::new();
        row.insert("user_id", Value::Uuid(id));
        row.insert("user_name", Value::Text("carol".into()));
        conn.put_row("User", row.clone()).unwrap();
        drop(conn);
        let reopened = Connection::open(&path).unwrap();
        let fetched = reopened
            .get_row_by_unique("User", "user_name", &Value::Text("carol".into()))
            .unwrap()
            .unwrap();
        assert_eq!(fetched, row);
        assert!(reopened.delete_row("User", &Value::Uuid(id)).unwrap());
        assert_eq!(
            reopened
                .get_row_by_unique("User", "user_name", &Value::Text("carol".into()))
                .unwrap(),
            None
        );
        drop(reopened);
        cleanup(&path);
    }

    #[test]
    fn unique_field_rejects_duplicates() {
        let conn = Connection::open_in_memory().unwrap();
        conn.ensure_schema(
            1,
            r#"
            table User {
                @@user_id: uuid,
                @user_name: utf8,
            }
            "#,
        )
        .unwrap();
        let a = uuid::Uuid::parse_str("00000000-0000-0000-0000-00000000000a").unwrap();
        let b = uuid::Uuid::parse_str("00000000-0000-0000-0000-00000000000b").unwrap();
        let mut row_a = Row::new();
        row_a.insert("user_id", Value::Uuid(a));
        row_a.insert("user_name", Value::Text("alice".into()));
        conn.put_row("User", row_a).unwrap();
        let mut row_b = Row::new();
        row_b.insert("user_id", Value::Uuid(b));
        row_b.insert("user_name", Value::Text("alice".into()));
        assert!(matches!(
            conn.put_row("User", row_b),
            Err(Error::Schema { message, .. }) if message.contains("unique")
        ));
        let got = conn
            .get_row_by_unique("User", "user_name", &Value::Text("alice".into()))
            .unwrap()
            .unwrap();
        assert_eq!(got.get("user_id"), Some(&Value::Uuid(a)));
    }

    #[test]
    fn transaction_commit_and_rollback() {
        let path = temp_db("txn");
        let conn = Connection::open(&path).unwrap();
        conn.begin().unwrap();
        conn.put("a", b"1").unwrap();
        conn.put("b", b"2").unwrap();
        assert!(conn.in_transaction());
        conn.commit().unwrap();
        assert!(!conn.in_transaction());
        drop(conn);

        let reopened = Connection::open(&path).unwrap();
        assert_eq!(reopened.get("a").unwrap(), Some(b"1".to_vec()));
        assert_eq!(reopened.get("b").unwrap(), Some(b"2".to_vec()));

        reopened.begin().unwrap();
        reopened.put("c", b"3").unwrap();
        assert_eq!(reopened.get("c").unwrap(), Some(b"3".to_vec()));
        reopened.rollback().unwrap();
        assert_eq!(reopened.get("c").unwrap(), None);
        assert!(matches!(
            reopened.begin().and_then(|_| reopened.begin()),
            Err(Error::Transaction { .. })
        ));
        // First `begin` succeeded; discard that draft before asserting commit-without-txn.
        reopened.rollback().unwrap();
        assert!(matches!(reopened.commit(), Err(Error::Transaction { .. })));
        cleanup(&path);
    }

    #[test]
    fn transaction_wraps_typed_rows() {
        let conn = Connection::open_in_memory().unwrap();
        conn.ensure_schema(
            1,
            r#"
            table Note {
                @@id: uuid,
                @title: utf8,
            }
            "#,
        )
        .unwrap();
        let id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap();
        conn.begin().unwrap();
        let mut row = Row::new();
        row.insert("id", Value::Uuid(id));
        row.insert("title", Value::Text("draft".into()));
        conn.put_row("Note", row).unwrap();
        conn.rollback().unwrap();
        assert_eq!(conn.get_row("Note", &Value::Uuid(id)).unwrap(), None);

        conn.begin().unwrap();
        let mut row = Row::new();
        row.insert("id", Value::Uuid(id));
        row.insert("title", Value::Text("saved".into()));
        conn.put_row("Note", row.clone()).unwrap();
        conn.commit().unwrap();
        assert_eq!(conn.get_row("Note", &Value::Uuid(id)).unwrap(), Some(row));
    }

    #[test]
    fn vos_object_ops_insert_filter_map_collect() {
        let conn = Connection::open_in_memory().unwrap();
        conn.ensure_schema(
            1,
            r#"
            table User {
                @@user_id: uuid,
                @user_name: utf8,
                active: bool,
            }
            "#,
        )
        .unwrap();
        conn.query(
            r#"
            User {
                user_id: uuid(),
                user_name: "alice",
                active: true,
            }.insert()
            "#,
        )
        .unwrap();
        conn.query(
            r#"
            User {
                user_id: uuid(),
                user_name: "bob",
                active: false,
            }.insert()
            "#,
        )
        .unwrap();
        let rows = conn
            .query(
                r#"
                User
                    .filter(x => x.active)
                    .map(x => x.{
                        user_id,
                        name: user_name,
                    })
                    .collect()
                "#,
            )
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("name"), Some(&Value::Text("alice".into())));
    }

    #[test]
    fn vos_object_ops_let_plan_and_now_update() {
        let conn = Connection::open_in_memory().unwrap();
        conn.ensure_schema(
            1,
            r#"
            table User {
                @@user_id: uuid,
                @user_name: utf8,
                active: bool,
                disabled_at: datetime?,
            }
            "#,
        )
        .unwrap();
        conn.query(
            r#"
            User {
                user_id: uuid(),
                user_name: "idle",
                active: false,
                disabled_at: null,
            }.insert()
            "#,
        )
        .unwrap();

        let updated = conn
            .query(
                r#"
                let inactive = User.filter(x => !x.active)
                inactive.update({
                    disabled_at: now(),
                })
                "#,
            )
            .unwrap();
        assert_eq!(updated.len(), 1);
        assert!(matches!(
            updated[0].get("disabled_at"),
            Some(Value::I64(n)) if *n > 0
        ));

        let counted = conn
            .query(r#"User.count(x => x.disabled_at != null)"#)
            .unwrap();
        assert_eq!(counted[0].get("value"), Some(&Value::I64(1)));
    }

    #[test]
    fn vos_object_ops_get_and_sort_take() {
        let conn = Connection::open_in_memory().unwrap();
        conn.ensure_schema(
            1,
            r#"
            table Note {
                @@id: uuid,
                title: utf8,
            }
            "#,
        )
        .unwrap();
        let a = uuid::Uuid::parse_str("00000000-0000-0000-0000-0000000000a1").unwrap();
        let b = uuid::Uuid::parse_str("00000000-0000-0000-0000-0000000000b2").unwrap();
        for (id, title) in [(a, "zeta"), (b, "alpha")] {
            let mut row = Row::new();
            row.insert("id", Value::Uuid(id));
            row.insert("title", Value::Text(title.into()));
            conn.put_row("Note", row).unwrap();
        }

        let got = conn
            .query(r#"Note.get("00000000-0000-0000-0000-0000000000a1")"#)
            .unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].get("title"), Some(&Value::Text("zeta".into())));

        let page = conn
            .query(
                r#"
                Note
                    .sort_by(x => x.title)
                    .skip(0)
                    .take(1)
                    .collect()
                "#,
            )
            .unwrap();
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].get("title"), Some(&Value::Text("alpha".into())));
    }

    /// Leading `.filter(x => x.<pk|unique> == const)` uses index lookup (not scan).
    #[test]
    fn vos_object_ops_unique_eq_filter_and_delete() {
        let conn = Connection::open_in_memory().unwrap();
        conn.ensure_schema(
            1,
            r#"
            table User {
                @@user_id: uuid,
                @user_name: utf8,
            }
            "#,
        )
        .unwrap();
        conn.query(
            r#"
            [
                User { user_id: uuid(), user_name: "alice" },
                User { user_id: uuid(), user_name: "bob" },
            ].insert()
            "#,
        )
        .unwrap();

        let found = conn
            .query(r#"User.filter(x => x.user_name == "bob").collect()"#)
            .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].get("user_name"), Some(&Value::Text("bob".into())));

        let deleted = conn
            .query(r#"User.filter(x => x.user_name == "alice").delete()"#)
            .unwrap();
        assert_eq!(deleted[0].get("value"), Some(&Value::U64(1)));
        assert_eq!(
            conn.query(r#"User.count()"#).unwrap()[0].get("value"),
            Some(&Value::I64(1))
        );
        assert!(conn
            .get_row_by_unique("User", "user_name", &Value::Text("alice".into()))
            .unwrap()
            .is_none());
    }

    #[test]
    fn rust_scalar_udf_create_and_call() {
        let conn = Connection::open_in_memory().unwrap();
        conn.create_scalar("double", 1, |args| match args {
            [Value::I64(n)] => Ok(Value::I64(n * 2)),
            _ => Err(Error::Udf {
                name: "double".into(),
                message: "expected i64".into(),
            }),
        })
        .unwrap();
        assert_eq!(
            conn.call_scalar("double", &[Value::I64(21)]).unwrap(),
            Value::I64(42)
        );
        assert!(matches!(
            conn.call_scalar("double", &[]),
            Err(Error::UdfArity {
                expected: 1,
                got: 0,
                ..
            })
        ));
        assert_eq!(conn.list_scalars(), vec!["double".to_owned()]);
        conn.remove_scalar("double").unwrap();
        assert!(matches!(
            conn.call_scalar("double", &[Value::I64(1)]),
            Err(Error::UdfNotFound { .. })
        ));
    }

    #[test]
    fn wal_and_shm_sidecars_recover_without_checkpoint() {
        let path = temp_db("wal");
        cleanup(&path);
        // Snapshot-era WAL coverage (paged has dedicated tests below).
        let conn = Connection::open_with_flags(&path, OpenFlags::create_snapshot_wal()).unwrap();
        assert!(!conn.is_paged());
        assert_eq!(conn.journal_mode(), JournalMode::Wal);
        assert!(conn
            .wal_path()
            .unwrap()
            .to_string_lossy()
            .ends_with(".yydb-wal"));
        assert!(conn
            .shm_path()
            .unwrap()
            .to_string_lossy()
            .ends_with(".yydb-shm"));
        conn.ensure_schema(1, "table T { @@id: uuid }").unwrap();
        conn.put("a", b"1").unwrap();
        conn.put("b", b"2").unwrap();
        assert!(conn.wal_frame_count().unwrap() >= 2);
        assert!(wal_path(&path).exists());
        assert!(shm_path(&path).exists());
        drop(conn);

        // Main file still has the empty/default image; recovery must come from WAL.
        let reopened = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        assert!(!reopened.is_paged());
        assert_eq!(reopened.get("a").unwrap(), Some(b"1".to_vec()));
        assert_eq!(reopened.get("b").unwrap(), Some(b"2".to_vec()));
        reopened.checkpoint().unwrap();
        assert_eq!(reopened.wal_frame_count().unwrap(), 0);
        drop(reopened);

        let after_ckpt = Connection::open(&path).unwrap();
        assert!(!after_ckpt.is_paged());
        assert_eq!(after_ckpt.get("b").unwrap(), Some(b"2".to_vec()));
        cleanup(&path);
    }

    #[test]
    fn wal_truncated_tail_recovers_last_complete_frame() {
        let path = temp_db("wal-trunc");
        cleanup(&path);
        let conn = Connection::open_with_flags(&path, OpenFlags::create_snapshot_wal()).unwrap();
        conn.put("a", b"1").unwrap();
        conn.put("b", b"2").unwrap();
        let frames_before = conn.wal_frame_count().unwrap();
        assert!(frames_before >= 2);
        drop(conn);

        // Simulate crash: append a torn frame (type + partial length).
        {
            use std::fs::OpenOptions;
            use std::io::Write;
            let mut file = OpenOptions::new()
                .append(true)
                .open(wal_path(&path))
                .unwrap();
            file.write_all(&[1, 0x10, 0x00]).unwrap(); // type=1, incomplete u32 len
            file.flush().unwrap();
        }

        let reopened = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        assert_eq!(reopened.get("a").unwrap(), Some(b"1".to_vec()));
        assert_eq!(reopened.get("b").unwrap(), Some(b"2".to_vec()));
        assert_eq!(reopened.wal_frame_count().unwrap(), frames_before);
        drop(reopened);
        cleanup(&path);
    }

    #[test]
    fn set_journal_mode_wal_to_delete_removes_sidecars() {
        let path = temp_db("mode");
        cleanup(&path);
        let conn = Connection::open(&path).unwrap();
        conn.set_journal_mode(JournalMode::Wal).unwrap();
        conn.put("k", b"v").unwrap();
        assert!(wal_path(&path).exists());
        conn.set_journal_mode(JournalMode::Delete).unwrap();
        assert!(!wal_path(&path).exists());
        assert!(!shm_path(&path).exists());
        assert_eq!(conn.get("k").unwrap(), Some(b"v".to_vec()));
        cleanup(&path);
    }

    #[test]
    fn paged_wal_recovers_put_and_schema_without_checkpoint() {
        let path = temp_db("paged-wal");
        let conn =
            Connection::open_with_flags(&path, OpenFlags::create_paged().with_wal()).unwrap();
        assert!(conn.is_paged());
        assert_eq!(conn.journal_mode(), JournalMode::Wal);
        conn.ensure_schema(1, "table T { @@id: uuid }").unwrap();
        conn.put("a", b"1").unwrap();
        conn.put("b", b"2").unwrap();
        assert!(conn.wal_frame_count().unwrap() >= 2);
        assert!(wal_path(&path).exists());
        drop(conn);

        // Main paged file still has the empty catalog image; truth is in WAL.
        let reopened = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        assert!(reopened.is_paged());
        assert_eq!(reopened.get("a").unwrap(), Some(b"1".to_vec()));
        assert_eq!(reopened.get("b").unwrap(), Some(b"2".to_vec()));
        assert!(reopened.schema().unwrap().is_some());
        assert!(reopened.wal_frame_count().unwrap() >= 2);
        drop(reopened);
        cleanup(&path);
    }

    #[test]
    fn paged_wal_truncated_tail_soft_repairs() {
        let path = temp_db("paged-wal-trunc");
        let conn = Connection::open_with_flags(&path, OpenFlags::create_paged_wal()).unwrap();
        conn.put("a", b"1").unwrap();
        conn.put("b", b"2").unwrap();
        let frames_before = conn.wal_frame_count().unwrap();
        assert!(frames_before >= 2);
        drop(conn);

        {
            use std::fs::OpenOptions;
            use std::io::Write;
            let mut file = OpenOptions::new()
                .append(true)
                .open(wal_path(&path))
                .unwrap();
            file.write_all(&[1, 0x10, 0x00]).unwrap();
            file.flush().unwrap();
        }

        let reopened = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        assert!(reopened.is_paged());
        assert_eq!(reopened.get("a").unwrap(), Some(b"1".to_vec()));
        assert_eq!(reopened.get("b").unwrap(), Some(b"2".to_vec()));
        assert_eq!(reopened.wal_frame_count().unwrap(), frames_before);
        drop(reopened);
        cleanup(&path);
    }

    #[test]
    fn paged_wal_checkpoint_folds_into_main_and_clears_frames() {
        let path = temp_db("paged-wal-ckpt");
        cleanup(&path);
        let conn =
            Connection::open_with_flags(&path, OpenFlags::create_paged().with_wal()).unwrap();
        conn.ensure_schema(1, "table T { @@id: uuid }").unwrap();
        conn.put("k", b"v").unwrap();
        assert!(conn.wal_frame_count().unwrap() >= 1);
        conn.checkpoint().unwrap();
        assert_eq!(conn.wal_frame_count().unwrap(), 0);
        drop(conn);

        // Open without WAL: main paged file must hold the folded state.
        let after = Connection::open(&path).unwrap();
        assert!(after.is_paged());
        assert_eq!(after.journal_mode(), JournalMode::Delete);
        assert_eq!(after.get("k").unwrap(), Some(b"v".to_vec()));
        assert!(after.schema().unwrap().is_some());
        assert_eq!(after.wal_frame_count().unwrap(), 0);
        drop(after);
        cleanup(&path);
    }

    /// Crash point: header-only (empty) WAL — reopen must keep main file state.
    #[test]
    fn wal_header_only_empty_frames_recovers_main() {
        let path = temp_db("wal-empty");
        cleanup(&path);
        let conn = Connection::open(&path).unwrap();
        assert!(conn.is_paged());
        conn.put("k", b"main").unwrap();
        conn.set_journal_mode(JournalMode::Wal).unwrap();
        assert_eq!(conn.wal_frame_count().unwrap(), 0);
        assert!(wal_path(&path).exists());
        drop(conn);

        let reopened = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        assert_eq!(reopened.get("k").unwrap(), Some(b"main".to_vec()));
        assert_eq!(reopened.wal_frame_count().unwrap(), 0);
        drop(reopened);
        cleanup(&path);
    }

    /// Crash point: after checkpoint, deleting `-wal`/`-shm` must not lose data.
    #[test]
    fn checkpoint_then_delete_sidecars_reopen_ok() {
        let path = temp_db("ckpt-rm-sidecars");
        cleanup(&path);
        let conn = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        assert!(conn.is_paged());
        conn.ensure_schema(1, "table T { @@id: uuid }").unwrap();
        conn.put("a", b"1").unwrap();
        conn.checkpoint().unwrap();
        assert_eq!(conn.wal_frame_count().unwrap(), 0);
        drop(conn);

        let _ = fs::remove_file(wal_path(&path));
        let _ = fs::remove_file(shm_path(&path));
        assert!(!wal_path(&path).exists());
        assert!(!shm_path(&path).exists());

        let reopened = Connection::open(&path).unwrap();
        assert!(reopened.is_paged());
        assert_eq!(reopened.get("a").unwrap(), Some(b"1".to_vec()));
        assert!(reopened.schema().unwrap().is_some());
        drop(reopened);
        cleanup(&path);
    }

    /// Crash point: snapshot WAL checkpoint then remove sidecars (compat path).
    #[test]
    fn snapshot_wal_checkpoint_then_delete_sidecars_reopen_ok() {
        let path = temp_db("snap-ckpt-rm");
        cleanup(&path);
        let conn = Connection::open_with_flags(&path, OpenFlags::create_snapshot_wal()).unwrap();
        assert!(!conn.is_paged());
        conn.put("s", b"snap").unwrap();
        conn.checkpoint().unwrap();
        drop(conn);

        let _ = fs::remove_file(wal_path(&path));
        let _ = fs::remove_file(shm_path(&path));

        let reopened = Connection::open(&path).unwrap();
        assert!(!reopened.is_paged());
        assert_eq!(reopened.get("s").unwrap(), Some(b"snap".to_vec()));
        drop(reopened);
        cleanup(&path);
    }

    /// Process-local write lease: open txn on A → B's put/begin are Busy.
    #[test]
    fn second_connection_busy_while_txn_open() {
        let path = temp_db("busy-txn");
        cleanup(&path);
        let a = Connection::open(&path).unwrap();
        let b = Connection::open(&path).unwrap();
        a.begin().unwrap();
        a.put("k", b"a").unwrap();
        assert!(b.put("k", b"b").unwrap_err().is_busy());
        assert!(b.begin().unwrap_err().is_busy());
        a.commit().unwrap();
        b.put("k", b"b").unwrap();
        assert_eq!(b.get("k").unwrap(), Some(b"b".to_vec()));
        drop(a);
        drop(b);
        cleanup(&path);
    }

    /// Autocommit short lease does not stick; peer can write after put returns.
    #[test]
    fn short_lease_released_after_autocommit_put() {
        let path = temp_db("busy-short");
        cleanup(&path);
        let a = Connection::open(&path).unwrap();
        let b = Connection::open(&path).unwrap();
        a.put("k", b"1").unwrap();
        b.put("k", b"2").unwrap();
        assert_eq!(a.get("k").unwrap(), Some(b"2".to_vec()));
        drop(a);
        drop(b);
        cleanup(&path);
    }

    /// Dropping a connection that holds a long lease frees the peer.
    #[test]
    fn drop_releases_long_write_lease() {
        let path = temp_db("busy-drop");
        cleanup(&path);
        let a = Connection::open(&path).unwrap();
        let b = Connection::open(&path).unwrap();
        a.begin().unwrap();
        assert!(b.put("k", b"x").unwrap_err().is_busy());
        drop(a);
        b.put("k", b"x").unwrap();
        drop(b);
        cleanup(&path);
    }

    /// Peer get is Busy while another connection holds a data txn (no torn read).
    #[test]
    fn peer_get_busy_while_writer_txn_open() {
        let path = temp_db("busy-read");
        cleanup(&path);
        let a = Connection::open(&path).unwrap();
        let b = Connection::open(&path).unwrap();
        a.put("k", b"before").unwrap();
        a.begin().unwrap();
        a.put("k", b"draft").unwrap();
        let err = b.get("k").unwrap_err();
        assert!(err.is_busy());
        assert!(err.to_string().contains("write lease"));
        a.commit().unwrap();
        assert_eq!(b.get("k").unwrap(), Some(b"draft".to_vec()));
        drop(a);
        drop(b);
        cleanup(&path);
    }

    /// After peer commit, reader sees updated btree pages (meta refresh).
    #[test]
    fn peer_sees_committed_put_after_lease_release() {
        let path = temp_db("peer-coherent");
        cleanup(&path);
        let a = Connection::open(&path).unwrap();
        let b = Connection::open(&path).unwrap();
        a.put("k", b"v1").unwrap();
        assert_eq!(b.get("k").unwrap(), Some(b"v1".to_vec()));
        a.put("k", b"v2").unwrap();
        assert_eq!(b.get("k").unwrap(), Some(b"v2".to_vec()));
        drop(a);
        drop(b);
        cleanup(&path);
    }

    /// Cross-process OS lock: foreign holder of `-write.lock` → Busy on put.
    #[test]
    fn os_write_lock_busy_when_foreign_holder() {
        use fs4::fs_std::FileExt;
        use std::fs::OpenOptions;

        let path = temp_db("os-lock");
        cleanup(&path);
        let conn = Connection::open(&path).unwrap();
        let lock = concurrency::write_lock_path(&path);
        let foreign = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&lock)
            .unwrap();
        assert!(foreign.try_lock_exclusive().unwrap());
        let err = conn.put("k", b"v").unwrap_err();
        assert!(err.is_busy());
        assert!(err.to_string().contains("another process"));
        drop(foreign);
        conn.put("k", b"v").unwrap();
        drop(conn);
        cleanup(&path);
    }

    /// Crash point: frame header complete (`type` + `u32 len`) but payload torn.
    #[test]
    fn paged_wal_torn_mid_body_soft_repairs() {
        let path = temp_db("paged-wal-torn-body");
        cleanup(&path);
        let conn = Connection::open_with_flags(&path, OpenFlags::create_paged_wal()).unwrap();
        conn.put("a", b"1").unwrap();
        conn.put("b", b"2").unwrap();
        let frames_before = conn.wal_frame_count().unwrap();
        assert!(frames_before >= 2);
        drop(conn);

        {
            use std::fs::OpenOptions;
            use std::io::Write;
            let mut file = OpenOptions::new()
                .append(true)
                .open(wal_path(&path))
                .unwrap();
            // type=1, claims 1000-byte payload, only 40 bytes follow.
            file.write_all(&[1]).unwrap();
            file.write_all(&1000u32.to_le_bytes()).unwrap();
            file.write_all(&[0xABu8; 40]).unwrap();
            file.flush().unwrap();
        }

        let reopened = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        assert!(reopened.is_paged());
        assert_eq!(reopened.get("a").unwrap(), Some(b"1".to_vec()));
        assert_eq!(reopened.get("b").unwrap(), Some(b"2".to_vec()));
        assert_eq!(reopened.wal_frame_count().unwrap(), frames_before);
        // Soft-repair must have truncated the torn body off the WAL.
        let wal_len = fs::metadata(wal_path(&path)).unwrap().len();
        assert_eq!(
            wal_len,
            journal::read_shm(&shm_path(&path)).unwrap().wal_bytes
        );
        drop(reopened);
        cleanup(&path);
    }

    /// Crash point: checkpoint wrote main, then crash before truncating WAL.
    ///
    /// Simulate by checkpointing, then restoring the pre-truncate `-wal`/`-shm`.
    /// Reopen must remain at the folded state (last complete frame wins).
    #[test]
    fn mid_checkpoint_main_folded_wal_retained_reopens_consistent() {
        let path = temp_db("mid-ckpt");
        cleanup(&path);
        let conn = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        assert!(conn.is_paged());
        conn.ensure_schema(1, "table T { @@id: uuid }").unwrap();
        conn.put("a", b"old").unwrap();
        conn.put("a", b"new").unwrap();
        let frames_before = conn.wal_frame_count().unwrap();
        assert!(frames_before >= 2);
        let wal_backup = fs::read(wal_path(&path)).unwrap();
        let shm_backup = fs::read(shm_path(&path)).unwrap();
        conn.checkpoint().unwrap();
        assert_eq!(conn.wal_frame_count().unwrap(), 0);
        drop(conn);

        // Crash window: main already folded; sidecars still look pre-truncate.
        fs::write(wal_path(&path), &wal_backup).unwrap();
        fs::write(shm_path(&path), &shm_backup).unwrap();
        assert!(wal_path(&path).exists());
        assert!(frames_before >= 2);

        let reopened = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        assert!(reopened.is_paged());
        assert_eq!(reopened.get("a").unwrap(), Some(b"new".to_vec()));
        assert!(reopened.schema().unwrap().is_some());
        // WAL frames may still be present; truth is last complete frame == main.
        assert_eq!(reopened.wal_frame_count().unwrap(), frames_before);
        reopened.checkpoint().unwrap();
        assert_eq!(reopened.wal_frame_count().unwrap(), 0);
        drop(reopened);

        let after = Connection::open(&path).unwrap();
        assert_eq!(after.get("a").unwrap(), Some(b"new".to_vec()));
        drop(after);
        cleanup(&path);
    }

    /// Crash point: complete frames then junk bytes — soft-repair to last good.
    #[test]
    fn paged_wal_junk_after_last_good_soft_repairs() {
        let path = temp_db("paged-wal-junk");
        cleanup(&path);
        let conn = Connection::open_with_flags(&path, OpenFlags::create_paged_wal()).unwrap();
        conn.put("a", b"1").unwrap();
        conn.put("b", b"2").unwrap();
        let frames_before = conn.wal_frame_count().unwrap();
        assert!(frames_before >= 2);
        let wal_bytes_before = fs::metadata(wal_path(&path)).unwrap().len();
        drop(conn);

        {
            use std::fs::OpenOptions;
            use std::io::Write;
            let mut file = OpenOptions::new()
                .append(true)
                .open(wal_path(&path))
                .unwrap();
            // Non-type-1 garbage after complete frames (dirty tail / torn header).
            file.write_all(&[0xFF, 0x00, 0xDE, 0xAD, 0xBE, 0xEF])
                .unwrap();
            file.flush().unwrap();
        }

        let reopened = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        assert!(reopened.is_paged());
        assert_eq!(reopened.get("a").unwrap(), Some(b"1".to_vec()));
        assert_eq!(reopened.get("b").unwrap(), Some(b"2".to_vec()));
        assert_eq!(reopened.wal_frame_count().unwrap(), frames_before);
        let wal_len = fs::metadata(wal_path(&path)).unwrap().len();
        assert_eq!(wal_len, wal_bytes_before);
        assert_eq!(
            wal_len,
            journal::read_shm(&shm_path(&path)).unwrap().wal_bytes
        );
        drop(reopened);
        cleanup(&path);
    }

    /// Crash point: SHM metadata ahead of durable WAL (append landed, SHM not).
    #[test]
    fn wal_shm_stale_high_resyncs_on_reopen() {
        let path = temp_db("shm-stale");
        cleanup(&path);
        let conn = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        conn.put("k", b"v").unwrap();
        let frames = conn.wal_frame_count().unwrap();
        assert!(frames >= 1);
        let wal_bytes = fs::metadata(wal_path(&path)).unwrap().len();
        drop(conn);

        // Inflate SHM as if a later frame had been announced but never written.
        journal::write_shm(
            &shm_path(&path),
            journal::ShmHeader {
                n_frames: frames.saturating_add(9),
                wal_bytes: wal_bytes.saturating_add(4096),
            },
        )
        .unwrap();

        let reopened = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        assert_eq!(reopened.get("k").unwrap(), Some(b"v".to_vec()));
        assert_eq!(reopened.wal_frame_count().unwrap(), frames);
        let shm = journal::read_shm(&shm_path(&path)).unwrap();
        assert_eq!(shm.n_frames, frames);
        assert_eq!(shm.wal_bytes, wal_bytes);
        drop(reopened);
        cleanup(&path);
    }

    /// Crash point: paged main truncated mid-write while type-1 WAL is complete.
    ///
    /// Truth is rebuilt from the last WAL snapshot; main is rewritten on reopen.
    #[test]
    fn paged_torn_main_truncated_recovers_from_wal() {
        let path = temp_db("torn-main-trunc");
        cleanup(&path);
        let conn = Connection::open_with_flags(&path, OpenFlags::create_paged_wal()).unwrap();
        conn.ensure_schema(1, "table T { @@id: uuid }").unwrap();
        conn.put("a", b"1").unwrap();
        conn.put("b", b"2").unwrap();
        let frames = conn.wal_frame_count().unwrap();
        assert!(frames >= 2);
        drop(conn);

        // Simulate crash during checkpoint / main rewrite: keep WAL, truncate main.
        let meta = fs::metadata(&path).unwrap();
        assert!(meta.len() > 64);
        {
            use std::fs::OpenOptions;
            let file = OpenOptions::new().write(true).open(&path).unwrap();
            file.set_len(64).unwrap();
        }
        assert!(fs::metadata(&path).unwrap().len() < pager::PAGE_SIZE as u64);

        let reopened = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        assert!(reopened.is_paged());
        assert_eq!(reopened.get("a").unwrap(), Some(b"1".to_vec()));
        assert_eq!(reopened.get("b").unwrap(), Some(b"2".to_vec()));
        assert!(reopened.schema().unwrap().is_some());
        assert!(reopened.wal_frame_count().unwrap() >= 2);
        // Main must have been rewritten to a full page image.
        assert!(fs::metadata(&path).unwrap().len() >= pager::PAGE_SIZE as u64);
        drop(reopened);
        cleanup(&path);
    }

    /// Crash point: paged main page checksum corrupt while WAL frames are good.
    #[test]
    fn paged_torn_main_checksum_corrupt_recovers_from_wal() {
        let path = temp_db("torn-main-crc");
        cleanup(&path);
        let conn = Connection::open_with_flags(&path, OpenFlags::create_paged_wal()).unwrap();
        conn.put("k", b"alive").unwrap();
        assert!(conn.wal_frame_count().unwrap() >= 1);
        drop(conn);

        let mut bytes = fs::read(&path).unwrap();
        // Flip a payload byte on page 0 so CRC fails at Pager::open.
        bytes[16] ^= 0xff;
        fs::write(&path, &bytes).unwrap();

        let reopened = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        assert!(reopened.is_paged());
        assert_eq!(reopened.get("k").unwrap(), Some(b"alive".to_vec()));
        drop(reopened);
        cleanup(&path);
    }

    /// Product rule: CAS `objects/` is not fenced with main/WAL commit.
    ///
    /// Orphan objects (bytes on disk, no committed row/KV referencing them) must
    /// **not** be invented as committed records on reopen.
    #[test]
    fn cas_orphan_object_without_commit_is_not_accepted_as_row() {
        let path = temp_db("cas-orphan");
        cleanup(&path);
        let conn = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        conn.ensure_schema(1, "table T { @@id: uuid }").unwrap();
        conn.put("committed", b"yes").unwrap();
        let object = conn.put_chunk(ObjectKind::Blob, b"orphan-bytes").unwrap();
        let object_path = conn.objects().path_for(&object);
        assert!(object_path.exists());
        // Crash before any DB commit that would reference `object`.
        drop(conn);

        let reopened = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        assert_eq!(reopened.get("committed").unwrap(), Some(b"yes".to_vec()));
        // Orphan CAS file may still exist (not rolled back); it is not a row.
        assert!(reopened.objects().path_for(&object).exists());
        assert_eq!(
            reopened.get(&format!("row/{}", object.hash_hex())).unwrap(),
            None
        );
        assert_eq!(reopened.record_count().unwrap(), 1);
        // Direct CAS read still works — bytes store ≠ committed database truth.
        assert_eq!(&*reopened.get_object(&object).unwrap(), b"orphan-bytes");
        drop(reopened);
        cleanup(&path);
    }

    /// Product rule (reverse skew): a committed pointer without CAS bytes stays a
    /// committed row; reading the object fails with `ObjectNotFound` (no silent fill).
    #[test]
    fn cas_missing_bytes_with_committed_pointer_fails_object_read() {
        let path = temp_db("cas-missing");
        cleanup(&path);
        let conn = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        let object = conn.put_chunk(ObjectKind::Blob, b"payload").unwrap();
        let hex = object.hash_hex();
        // Commit a KV pointer (row ObjectRef encoding is not required for this rule).
        conn.put(&format!("blob/{hex}"), hex.as_bytes()).unwrap();
        let object_path = conn.objects().path_for(&object);
        assert!(object_path.exists());
        drop(conn);

        fs::remove_file(&object_path).unwrap();
        assert!(!object_path.exists());

        let reopened = Connection::open_with_flags(&path, OpenFlags::wal()).unwrap();
        assert_eq!(
            reopened.get(&format!("blob/{hex}")).unwrap(),
            Some(hex.as_bytes().to_vec())
        );
        assert!(matches!(
            reopened.get_object(&object),
            Err(Error::ObjectNotFound { .. })
        ));
        drop(reopened);
        cleanup(&path);
    }

    #[test]
    fn cas_objects_hash2_path_and_hot_cold() {
        let conn = Connection::open_in_memory().unwrap();
        let object = conn.put_chunk(ObjectKind::Blob, b"hello-cas").unwrap();
        let path = conn.objects().path_for(&object);
        let path_s = path.to_string_lossy().replace('\\', "/");
        assert!(path_s.contains("/objects/"));
        assert!(path_s.ends_with(".bytes"));
        let hex = object.hash_hex();
        assert!(path_s.contains(&format!("/{}/{}", &hex[..2], hex)));
        assert_eq!(&*conn.get_object(&object).unwrap(), b"hello-cas");
        assert_eq!(conn.pin_object(&object).unwrap(), Tier::Hot);
        assert_eq!(conn.object_tier(&object), Tier::Hot);
        assert_eq!(conn.evict_object(&object).unwrap(), Tier::Cold);
        assert_eq!(&*conn.get_object(&object).unwrap(), b"hello-cas");
    }

    #[test]
    fn chunked_file_range_read() {
        let conn = Connection::open_in_memory().unwrap();
        let payload: Vec<u8> = (0..2000u16).map(|v| (v % 256) as u8).collect();
        let manifest = conn
            .put_file_chunked(std::io::Cursor::new(payload.clone()), 500)
            .unwrap();
        assert!(manifest.chunks.len() >= 4);
        assert_eq!(manifest.total_size, 2000);
        let mid = conn.read_file_range(&manifest, 500, 500).unwrap();
        assert_eq!(mid, payload[500..1000]);
        let tail = conn.read_file_range(&manifest, 1800, 500).unwrap();
        assert_eq!(tail, payload[1800..]);
    }

    #[test]
    fn vector_via_cas_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        let vector = Vector::new(vec![1.0, 2.5, -3.0]).unwrap();
        let object = conn.put_vector(&vector).unwrap();
        assert_eq!(object.kind, ObjectKind::VectorPayload);
        let loaded = conn.get_vector(&object).unwrap();
        assert_eq!(loaded.dim, 3);
        assert_eq!(&*loaded.data, &*vector.data);
    }

    #[test]
    fn version_is_non_empty() {
        assert!(!version().is_empty());
    }

    #[test]
    fn ensure_schema_builds_field_catalog() {
        let conn = Connection::open_in_memory().unwrap();
        conn.ensure_schema(1, "table User { @@user_id: uuid, @user_name: utf8 }")
            .unwrap();
        let catalog = conn.field_catalog().unwrap().unwrap();
        assert_eq!(catalog.revisions.ddl, 1);
        let (id, slot) = catalog
            .type_entry("User")
            .unwrap()
            .resolve("user_name")
            .unwrap();
        assert_eq!(slot, 1);
        assert_eq!(id.0, 2);
        assert_eq!(conn.ddl_revision().unwrap(), 1);
    }

    #[test]
    fn ddl_session_rename_persists_field_id_across_reopen() {
        let path = temp_db("ddl-rename");
        let conn = Connection::open(&path).unwrap();
        conn.ensure_schema(
            1,
            r#"
            table User {
                @@user_id: uuid,
                @user_name: utf8,
                avatar: utf8?,
            }
            "#,
        )
        .unwrap();
        let before = conn
            .field_catalog()
            .unwrap()
            .unwrap()
            .type_entry("User")
            .unwrap()
            .resolve("user_name")
            .unwrap();

        let ddl = conn.begin_ddl_session().unwrap();
        let renamed = ddl
            .rename_field("User", "user_name", "display_name")
            .unwrap();
        assert_eq!(renamed, before);
        assert_eq!(ddl.ddl_revision().unwrap(), 2);
        ddl.commit().unwrap();

        let schema = conn.schema().unwrap().unwrap();
        assert!(schema.document.contains("display_name"));
        assert!(!schema.document.contains("user_name"));
        drop(conn);

        let reopened = Connection::open(&path).unwrap();
        assert_eq!(reopened.ddl_revision().unwrap(), 2);
        let after = reopened
            .field_catalog()
            .unwrap()
            .unwrap()
            .type_entry("User")
            .unwrap()
            .resolve("display_name")
            .unwrap();
        assert_eq!(after, before);
        assert!(reopened
            .field_catalog()
            .unwrap()
            .unwrap()
            .type_entry("User")
            .unwrap()
            .resolve("user_name")
            .is_err());
        cleanup(&path);
    }

    #[test]
    fn ddl_session_required_outside_session() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(conn.begin_ddl_session().is_err());
    }

    #[test]
    fn ddl_session_add_field_allocates_new_slot() {
        let conn = Connection::open_in_memory().unwrap();
        conn.ensure_schema(1, "table User { @@user_id: uuid, @user_name: utf8 }")
            .unwrap();
        let ddl = conn.begin_ddl_session().unwrap();
        let (_id, slot) = ddl
            .add_field(
                "User",
                "email",
                vos::ast::TypeExpr::Builtin(vos::ast::BuiltinType::Utf8),
                vec![],
            )
            .unwrap();
        assert_eq!(slot, 2);
        ddl.commit().unwrap();
        let schema = conn.schema().unwrap().unwrap();
        assert!(schema.document.contains("email: utf8"));
        let catalog = conn.field_catalog().unwrap().unwrap();
        assert_eq!(
            catalog
                .type_entry("User")
                .unwrap()
                .resolve("email")
                .unwrap()
                .1,
            2
        );
        assert_eq!(conn.ddl_revision().unwrap(), 2);
    }

    #[test]
    fn query_session_stale_after_ddl_commit() {
        let conn = Connection::open_in_memory().unwrap();
        conn.ensure_schema(1, "table User { @@user_id: uuid, @user_name: utf8 }")
            .unwrap();
        let query = conn.begin_query_session().unwrap();
        assert_eq!(query.created_ddl_revision(), 1);

        let ddl = conn.begin_ddl_session().unwrap();
        ddl.rename_field("User", "user_name", "display_name")
            .unwrap();
        ddl.commit().unwrap();

        let err = query.query("User.filter(x => true).collect()").unwrap_err();
        assert!(err.to_string().contains("VOS-SESSION-STALE"));

        let fresh = conn.begin_query_session().unwrap();
        assert_eq!(fresh.created_ddl_revision(), 2);
        fresh.check().unwrap();
    }

    #[test]
    fn data_txn_commit_fails_if_ddl_revision_diverges() {
        let conn = Connection::open_in_memory().unwrap();
        conn.ensure_schema(1, "table User { @@user_id: uuid }")
            .unwrap();
        conn.begin().unwrap();
        *conn
            .txn_ddl_revision
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some(0);
        let err = conn.commit().unwrap_err();
        assert!(err.to_string().contains("VOS-TRANSACTION-STALE-DDL"));
        conn.rollback().unwrap();
    }

    #[test]
    fn install_macro_rejects_star_and_blocks_drop() {
        let conn = Connection::open_in_memory().unwrap();
        conn.ensure_schema(1, "table User { @@user_id: uuid, @user_name: utf8 }")
            .unwrap();
        let ddl = conn.begin_ddl_session().unwrap();
        let err = ddl
            .install_macro(
                "snapshot",
                &[("user".into(), vos::ast::TypeExpr::Named("User".into()))],
                vos::ast::TypeExpr::Named("User".into()),
                "user.*",
            )
            .unwrap_err();
        assert!(err.to_string().contains("VOS-MACRO-DYNAMIC-FORBIDDEN"));

        ddl.install_macro(
            "public_name",
            &[("user".into(), vos::ast::TypeExpr::Named("User".into()))],
            vos::ast::TypeExpr::Builtin(vos::ast::BuiltinType::Utf8),
            "user.user_name",
        )
        .unwrap();
        let blocked = ddl.drop_field_checked("User", "user_name").unwrap_err();
        assert!(blocked.to_string().contains("VOS-DDL-FIELD-IN-USE"));
        assert!(blocked.to_string().contains("macro public_name"));
        ddl.commit().unwrap();

        let catalog = conn.field_catalog().unwrap().unwrap();
        let m = catalog.macro_entry("public_name").unwrap();
        assert_eq!(m.ir.field_loads.len(), 1);
        assert_eq!(m.ir.field_loads[0].virtual_field, 1);
    }

    #[test]
    fn rename_preserves_macro_semantic_hash() {
        let conn = Connection::open_in_memory().unwrap();
        conn.ensure_schema(1, "table User { @@user_id: uuid, @user_name: utf8 }")
            .unwrap();
        {
            let ddl = conn.begin_ddl_session().unwrap();
            ddl.install_macro(
                "public_name",
                &[("user".into(), vos::ast::TypeExpr::Named("User".into()))],
                vos::ast::TypeExpr::Builtin(vos::ast::BuiltinType::Utf8),
                "user.user_name",
            )
            .unwrap();
            ddl.commit().unwrap();
        }
        let before = conn
            .field_catalog()
            .unwrap()
            .unwrap()
            .macro_entry("public_name")
            .unwrap()
            .clone();
        {
            let ddl = conn.begin_ddl_session().unwrap();
            ddl.rename_field("User", "user_name", "display_name")
                .unwrap();
            ddl.commit().unwrap();
        }
        let after = conn
            .field_catalog()
            .unwrap()
            .unwrap()
            .macro_entry("public_name")
            .unwrap()
            .clone();
        assert_eq!(before.semantic_hash, after.semantic_hash);
        assert_eq!(before.ir.field_loads, after.ir.field_loads);
        assert!(after.source.contains("display_name"));
        assert!(!after.source.contains("user_name"));
        assert_ne!(before.source_hash, after.source_hash);
    }

    #[test]
    fn prepared_plan_stale_after_ddl() {
        let conn = Connection::open_in_memory().unwrap();
        conn.ensure_schema(1, "table User { @@user_id: uuid, @user_name: utf8 }")
            .unwrap();
        let plan = PreparedPlan::prepare(&conn, "User.filter(x => true).collect()").unwrap();
        let ddl = conn.begin_ddl_session().unwrap();
        ddl.rename_field("User", "user_name", "display_name")
            .unwrap();
        ddl.commit().unwrap();
        let err = plan.execute(&conn).unwrap_err();
        assert!(err.to_string().contains("VOS-PREPARED-STALE"));
    }
}
