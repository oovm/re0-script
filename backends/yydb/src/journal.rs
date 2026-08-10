//! Journal modes and `-wal` / `-shm` sidecar layout.
//!
//! Product surface remains one primary `.yydb` file. In [`JournalMode::Wal`],
//! durable writes append to `{path}-wal` and coordination metadata lives in
//! `{path}-shm`. Call [`crate::Connection::checkpoint`] to fold the WAL back
//! into the main file (snapshot rewrite or paged pages). Truncated WAL tails
//! are soft-repaired on reopen — see [`crate`] documentation and
//! `documentation/file-format.md`.
//!
//! Both snapshot (`YYDB\x01`) and paged (`YYDB\x02`) layouts share one WAL.
//! Frame types: **1** full-state snapshot, **2** sealed page image, **3**
//! page-batch commit barrier. Paged commits use type-2/3; snapshot-era uses
//! type-1. Soft-repair discards torn/junk tails and incomplete page batches.

use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::pager::{PageId, PAGE_SIZE};
use yydb_types::{Error, Result};

/// Full-state snapshot frame (`encode(State)`).
pub(crate) const WAL_FRAME_SNAPSHOT: u8 = 1;
/// Sealed page image (`page_id` + `PAGE_SIZE` bytes).
pub(crate) const WAL_FRAME_PAGE: u8 = 2;
/// End of a page-batch commit (empty payload).
pub(crate) const WAL_FRAME_PAGE_COMMIT: u8 = 3;

/// How durable writes are published to disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JournalMode {
    /// Rewrite the main database file on each commit (default).
    #[default]
    Delete,
    /// Append commits to `{db}-wal` and keep `{db}-shm` index metadata.
    Wal,
}

impl JournalMode {
    /// Parse a journal_mode name (`delete` / `wal`).
    pub fn parse(name: &str) -> Result<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "delete" => Ok(Self::Delete),
            "wal" => Ok(Self::Wal),
            _ => Err(Error::Unsupported("journal_mode (use delete|wal)")),
        }
    }

    /// Stable lowercase name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Delete => "delete",
            Self::Wal => "wal",
        }
    }
}

/// On-disk layout used when creating a new `.yydb` file.
///
/// Ignored when the path already exists (open dispatches by magic).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreateLayout {
    /// Snapshot-era `YYDB\x01` — use [`OpenFlags::create_snapshot`] for new files.
    Snapshot,
    /// Paged-era `YYDB\x02` — default for [`crate::Connection::open`].
    #[default]
    Paged,
}

/// Flags for [`crate::Connection::open_with_flags`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OpenFlags {
    /// Initial journal mode for this connection.
    pub journal_mode: JournalMode,
    /// Layout when creating a missing file. Existing files open by magic.
    pub create_layout: CreateLayout,
}

impl OpenFlags {
    /// Default flags (`delete` journal, paged create layout).
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable WAL + SHM sidecars (paged create layout by default).
    ///
    /// Existing files always open by magic. For snapshot-era create under WAL,
    /// use [`Self::create_snapshot_wal`] or [`Self::create_snapshot`] +
    /// [`Self::with_wal`].
    pub fn wal() -> Self {
        Self {
            journal_mode: JournalMode::Wal,
            create_layout: CreateLayout::Paged,
        }
    }

    /// Create a snapshot-era file when the path is missing (`delete` journal).
    ///
    /// Prefer this for tests/compat that need `YYDB\x01`. New product databases
    /// should use the default paged layout ([`Self::new`] / [`Self::wal`]).
    pub fn create_snapshot() -> Self {
        Self {
            journal_mode: JournalMode::Delete,
            create_layout: CreateLayout::Snapshot,
        }
    }

    /// Create a snapshot-era file with WAL sidecars when the path is missing.
    pub fn create_snapshot_wal() -> Self {
        Self {
            journal_mode: JournalMode::Wal,
            create_layout: CreateLayout::Snapshot,
        }
    }

    /// Create a paged-era file when the path is missing (`delete` journal).
    ///
    /// Equivalent to [`Self::new`] for create layout; kept for explicit call sites.
    pub fn create_paged() -> Self {
        Self {
            journal_mode: JournalMode::Delete,
            create_layout: CreateLayout::Paged,
        }
    }

    /// Create a paged-era file with WAL sidecars when the path is missing.
    ///
    /// Equivalent to [`Self::wal`] for create layout; kept for explicit call sites.
    pub fn create_paged_wal() -> Self {
        Self {
            journal_mode: JournalMode::Wal,
            create_layout: CreateLayout::Paged,
        }
    }

    /// Switch journal mode to WAL, keeping [`Self::create_layout`].
    pub fn with_wal(mut self) -> Self {
        self.journal_mode = JournalMode::Wal;
        self
    }
}

const WAL_MAGIC: &[u8] = b"YYWL\x01";
const SHM_MAGIC: &[u8] = b"YYSH\x01";
const SHM_BYTES: usize = 32;

/// `{path}-wal` companion path.
pub fn wal_path(db: &Path) -> PathBuf {
    sidecar(db, "-wal")
}

/// `{path}-shm` companion path.
pub fn shm_path(db: &Path) -> PathBuf {
    sidecar(db, "-shm")
}

fn sidecar(db: &Path, suffix: &str) -> PathBuf {
    let mut os = db.as_os_str().to_owned();
    os.push(suffix);
    PathBuf::from(os)
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ShmHeader {
    pub(crate) n_frames: u32,
    pub(crate) wal_bytes: u64,
}

pub(crate) fn ensure_wal_sidecars(db: &Path) -> Result<()> {
    let wal = wal_path(db);
    let shm = shm_path(db);
    if !wal.exists() {
        let mut file = File::create(&wal)?;
        file.write_all(WAL_MAGIC)?;
    }
    if !shm.exists() {
        write_shm(&shm, ShmHeader::default())?;
    }
    Ok(())
}

pub(crate) fn remove_wal_sidecars(db: &Path) -> Result<()> {
    let wal = wal_path(db);
    let shm = shm_path(db);
    if wal.exists() {
        fs::remove_file(&wal)?;
    }
    if shm.exists() {
        fs::remove_file(&shm)?;
    }
    Ok(())
}

pub(crate) fn read_shm(path: &Path) -> Result<ShmHeader> {
    if !path.exists() {
        return Ok(ShmHeader::default());
    }
    let bytes = fs::read(path)?;
    if bytes.len() < SHM_BYTES {
        return Err(Error::Corrupt("shm too short"));
    }
    if &bytes[..SHM_MAGIC.len()] != SHM_MAGIC {
        return Err(Error::Corrupt("unknown shm header"));
    }
    let n_frames = u32::from_le_bytes(bytes[5..9].try_into().unwrap());
    let wal_bytes = u64::from_le_bytes(bytes[9..17].try_into().unwrap());
    Ok(ShmHeader {
        n_frames,
        wal_bytes,
    })
}

pub(crate) fn write_shm(path: &Path, header: ShmHeader) -> Result<()> {
    let mut buf = vec![0_u8; SHM_BYTES];
    buf[..SHM_MAGIC.len()].copy_from_slice(SHM_MAGIC);
    buf[5..9].copy_from_slice(&header.n_frames.to_le_bytes());
    buf[9..17].copy_from_slice(&header.wal_bytes.to_le_bytes());
    fs::write(path, buf).map_err(Error::Io)
}

/// Append one full-state snapshot frame to the WAL and refresh SHM.
pub(crate) fn append_snapshot_frame(db: &Path, payload: &[u8]) -> Result<()> {
    append_frame(db, WAL_FRAME_SNAPSHOT, payload)
}

/// Append a durable page-batch commit: type-2 page images + type-3 barrier.
///
/// `pages` must already include checksum trailers (as stored by [`crate::pager::Pager`]).
pub(crate) fn append_page_batch(db: &Path, pages: &[(PageId, [u8; PAGE_SIZE])]) -> Result<()> {
    ensure_wal_sidecars(db)?;
    let wal = wal_path(db);
    let mut file = OpenOptions::new().append(true).open(&wal)?;
    let mut added = 0u32;
    for (id, data) in pages {
        // payload = page_id || sealed page bytes
        let mut payload = Vec::with_capacity(4 + PAGE_SIZE);
        payload.extend_from_slice(&id.to_le_bytes());
        payload.extend_from_slice(data);
        write_frame_to(&mut file, WAL_FRAME_PAGE, &payload)?;
        added = added.saturating_add(1);
    }
    write_frame_to(&mut file, WAL_FRAME_PAGE_COMMIT, &[])?;
    added = added.saturating_add(1);
    file.flush()?;

    let meta = file.metadata()?;
    let mut shm = read_shm(&shm_path(db))?;
    shm.n_frames = shm.n_frames.saturating_add(added);
    shm.wal_bytes = meta.len();
    write_shm(&shm_path(db), shm)?;
    Ok(())
}

fn append_frame(db: &Path, kind: u8, payload: &[u8]) -> Result<()> {
    ensure_wal_sidecars(db)?;
    let wal = wal_path(db);
    let mut file = OpenOptions::new().append(true).open(&wal)?;
    write_frame_to(&mut file, kind, payload)?;
    file.flush()?;

    let meta = file.metadata()?;
    let mut shm = read_shm(&shm_path(db))?;
    shm.n_frames = shm.n_frames.saturating_add(1);
    shm.wal_bytes = meta.len();
    write_shm(&shm_path(db), shm)?;
    Ok(())
}

fn write_frame_to(file: &mut File, kind: u8, payload: &[u8]) -> Result<()> {
    file.write_all(&[kind])?;
    let len = u32::try_from(payload.len()).map_err(|_| Error::Corrupt("wal frame too large"))?;
    file.write_all(&len.to_le_bytes())?;
    file.write_all(payload)?;
    Ok(())
}

/// Last durable WAL commit after soft-repair.
#[derive(Debug, Clone)]
pub(crate) enum WalCommit {
    /// Type-1 full-state payload.
    Snapshot(Vec<u8>),
    /// Complete type-2…type-3 page batch (sealed page images).
    Pages(Vec<(PageId, [u8; PAGE_SIZE])>),
}

/// Scan result for WAL frames (after soft-repair bookkeeping).
#[derive(Debug, Clone)]
pub(crate) struct WalScan {
    /// Last complete durable commit, if any.
    pub(crate) last_commit: Option<WalCommit>,
    /// Last complete type-1 payload (compat with snapshot merge helpers).
    pub(crate) last_payload: Option<Vec<u8>>,
    /// Number of complete frames kept.
    pub(crate) n_frames: u32,
    /// Byte offset of the durable prefix (includes magic).
    pub(crate) durable_bytes: u64,
}

/// Soft-repair rules:
/// - Torn length / mid-body / junk tails are discarded.
/// - Type-2 pages without a trailing type-3 are an incomplete batch and discarded.
/// - SHM is always resynced to the durable prefix.
pub(crate) fn scan_wal(db: &Path) -> Result<WalScan> {
    let wal = wal_path(db);
    if !wal.exists() {
        return Ok(WalScan {
            last_commit: None,
            last_payload: None,
            n_frames: 0,
            durable_bytes: 0,
        });
    }
    let bytes = fs::read(&wal)?;
    if bytes.is_empty() {
        return Ok(WalScan {
            last_commit: None,
            last_payload: None,
            n_frames: 0,
            durable_bytes: 0,
        });
    }
    if bytes.len() < WAL_MAGIC.len() || &bytes[..WAL_MAGIC.len()] != WAL_MAGIC {
        return Err(Error::Corrupt("unknown wal header"));
    }
    let mut cursor = WAL_MAGIC.len();
    let mut last_good = cursor;
    let mut frames = 0u32;
    let mut last_commit = None;
    let mut last_payload = None;
    let mut pending_pages: Vec<(PageId, [u8; PAGE_SIZE])> = Vec::new();
    let mut pending_start = cursor;

    while cursor < bytes.len() {
        let Some(&kind) = bytes.get(cursor) else {
            break;
        };
        if kind != WAL_FRAME_SNAPSHOT && kind != WAL_FRAME_PAGE && kind != WAL_FRAME_PAGE_COMMIT {
            // Junk / unsupported — soft-repair the tail.
            break;
        }
        let frame_at = cursor;
        cursor += 1;
        if cursor + 4 > bytes.len() {
            break;
        }
        let len = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap()) as usize;
        cursor += 4;
        if cursor + len > bytes.len() {
            break;
        }
        let payload = &bytes[cursor..cursor + len];
        cursor += len;

        match kind {
            WAL_FRAME_SNAPSHOT => {
                pending_pages.clear();
                let owned = payload.to_vec();
                last_payload = Some(owned.clone());
                last_commit = Some(WalCommit::Snapshot(owned));
                last_good = cursor;
                frames = frames.saturating_add(1);
                pending_start = cursor;
            }
            WAL_FRAME_PAGE => {
                if pending_pages.is_empty() {
                    pending_start = frame_at;
                }
                if payload.len() != 4 + PAGE_SIZE {
                    // Malformed page frame — stop before it.
                    break;
                }
                let id = u32::from_le_bytes(payload[..4].try_into().unwrap());
                let mut page = [0u8; PAGE_SIZE];
                page.copy_from_slice(&payload[4..]);
                pending_pages.push((id, page));
                // Page frames alone are not durable until type-3.
                frames = frames.saturating_add(1);
                // Do not advance last_good yet — incomplete batch is not durable.
            }
            WAL_FRAME_PAGE_COMMIT => {
                if len != 0 {
                    break;
                }
                last_commit = Some(WalCommit::Pages(std::mem::take(&mut pending_pages)));
                // Snapshot payload no longer the latest commit.
                last_payload = None;
                last_good = cursor;
                frames = frames.saturating_add(1);
                pending_start = cursor;
            }
            _ => break,
        }
    }

    // Incomplete page batch: roll back to before pending_start.
    if !pending_pages.is_empty() {
        // Drop non-durable page frames from the frame count.
        let pending_count = pending_pages.len() as u32;
        frames = frames.saturating_sub(pending_count);
        // last_good already points at previous durable commit.
        let _ = pending_start;
    }

    if last_good < bytes.len() {
        let mut file = OpenOptions::new().write(true).open(&wal)?;
        file.set_len(last_good as u64)?;
        file.flush()?;
    }
    write_shm(
        &shm_path(db),
        ShmHeader {
            n_frames: frames,
            wal_bytes: last_good as u64,
        },
    )?;
    Ok(WalScan {
        last_commit,
        last_payload,
        n_frames: frames,
        durable_bytes: last_good as u64,
    })
}

/// Compat wrapper: scan and expose type-1 fields used by snapshot helpers.
pub(crate) fn scan_wal_snapshots(db: &Path) -> Result<WalScan> {
    scan_wal(db)
}

/// Replay type-1 snapshot frames after `base` (later frames win).
///
/// Page-batch commits are ignored here — callers that need pages use
/// [`scan_wal`] / [`last_wal_commit`].
pub(crate) fn replay_wal_snapshots(db: &Path, base: Vec<u8>) -> Result<Vec<u8>> {
    let scan = scan_wal(db)?;
    match scan.last_commit {
        Some(WalCommit::Snapshot(payload)) => Ok(payload),
        _ => Ok(base),
    }
}

/// Last complete type-1 payload when the latest durable commit is a snapshot.
pub(crate) fn last_complete_wal_payload(db: &Path) -> Result<Option<Vec<u8>>> {
    match last_wal_commit(db)? {
        Some(WalCommit::Snapshot(payload)) => Ok(Some(payload)),
        _ => Ok(None),
    }
}

/// Last durable WAL commit (snapshot or page batch).
pub(crate) fn last_wal_commit(db: &Path) -> Result<Option<WalCommit>> {
    Ok(scan_wal(db)?.last_commit)
}

pub(crate) fn truncate_wal(db: &Path) -> Result<()> {
    let wal = wal_path(db);
    let mut file = File::create(&wal)?;
    file.write_all(WAL_MAGIC)?;
    write_shm(&shm_path(db), ShmHeader::default())?;
    Ok(())
}

pub(crate) fn wal_frame_count(db: &Path) -> Result<u32> {
    Ok(read_shm(&shm_path(db))?.n_frames)
}

/// Detect existing sidecars (useful for `info`).
pub fn sidecar_status(db: &Path) -> Result<(bool, bool, u32)> {
    let wal = wal_path(db);
    let shm = shm_path(db);
    let frames = if shm.exists() {
        read_shm(&shm)?.n_frames
    } else {
        0
    };
    Ok((wal.exists(), shm.exists(), frames))
}

#[allow(dead_code)]
pub(crate) fn read_exact_prefix(path: &Path, n: usize) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buf = vec![0_u8; n];
    file.read_exact(&mut buf)?;
    Ok(buf)
}
