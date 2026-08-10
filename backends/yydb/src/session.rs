//! Query / data sessions with `DdlRevision` freshness checks.
//!
//! Normative: `vos-language/docs/effects.md`. After a DDL commit advances
//! `DdlRevision`, sessions opened at an older revision fail with
//! `VOS-SESSION-STALE` on the next use.

use crate::macros::blake3_hex;
use crate::table::Row;
use crate::Connection;
use yydb_types::{Error, Result};

/// Read-oriented session (`PURE` / `DATA_READ` / `DDL_READ`).
pub struct QuerySession<'a> {
    conn: &'a Connection,
    created_ddl_revision: u64,
}

impl<'a> QuerySession<'a> {
    pub(crate) fn open(conn: &'a Connection) -> Result<Self> {
        Ok(Self {
            conn,
            created_ddl_revision: conn.ddl_revision()?,
        })
    }

    /// `DdlRevision` captured when this session was opened.
    pub fn created_ddl_revision(&self) -> u64 {
        self.created_ddl_revision
    }

    /// Fail with `VOS-SESSION-STALE` when the database DDL advanced.
    pub fn check(&self) -> Result<()> {
        check_session_fresh(self.conn, self.created_ddl_revision)
    }

    /// Parse and execute a VOS program after a freshness check.
    pub fn query(&self, vos_program: &str) -> Result<Vec<Row>> {
        self.check()?;
        self.conn.query(vos_program)
    }

    /// Prepare a plan bound to this session’s DDL revision.
    pub fn prepare(&self, vos_program: &str) -> Result<PreparedPlan> {
        self.check()?;
        PreparedPlan::prepare(self.conn, vos_program)
    }
}

/// Data session (`DATA_WRITE` allowed; `DDL_WRITE` forbidden).
pub struct DataSession<'a> {
    conn: &'a Connection,
    created_ddl_revision: u64,
}

impl<'a> DataSession<'a> {
    pub(crate) fn open(conn: &'a Connection) -> Result<Self> {
        Ok(Self {
            conn,
            created_ddl_revision: conn.ddl_revision()?,
        })
    }

    /// `DdlRevision` captured when this session was opened.
    pub fn created_ddl_revision(&self) -> u64 {
        self.created_ddl_revision
    }

    /// Fail with `VOS-SESSION-STALE` when the database DDL advanced.
    pub fn check(&self) -> Result<()> {
        check_session_fresh(self.conn, self.created_ddl_revision)
    }

    /// Parse and execute a VOS program after a freshness check.
    pub fn query(&self, vos_program: &str) -> Result<Vec<Row>> {
        self.check()?;
        self.conn.query(vos_program)
    }

    /// Begin a data transaction fenced to the current DDL revision.
    pub fn begin(&self) -> Result<()> {
        self.check()?;
        self.conn.begin()
    }

    /// Commit the open data transaction (also checks DDL fence).
    pub fn commit(&self) -> Result<()> {
        self.check()?;
        self.conn.commit()
    }

    /// Roll back the open data transaction.
    pub fn rollback(&self) -> Result<()> {
        self.conn.rollback()
    }
}

/// Prepared query plan — fails with `VOS-PREPARED-STALE` after DDL advances.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedPlan {
    ddl_revision: u64,
    source: String,
    source_hash: String,
}

impl PreparedPlan {
    /// Bind `vos_program` to the connection’s current `DdlRevision`.
    pub fn prepare(conn: &Connection, vos_program: &str) -> Result<Self> {
        vos::parser::parse_program(vos_program)
            .map_err(|d| crate::query::map_program_diagnostics(vos_program, d))?;
        Ok(Self {
            ddl_revision: conn.ddl_revision()?,
            source_hash: blake3_hex(vos_program.as_bytes()),
            source: vos_program.to_owned(),
        })
    }

    /// Revision this plan was prepared against.
    pub fn ddl_revision(&self) -> u64 {
        self.ddl_revision
    }

    /// BLAKE3 hex of the source text.
    pub fn source_hash(&self) -> &str {
        &self.source_hash
    }

    /// Execute if the database DDL revision still matches.
    pub fn execute(&self, conn: &Connection) -> Result<Vec<Row>> {
        let current = conn.ddl_revision()?;
        if self.ddl_revision != current {
            return Err(prepared_stale(self.ddl_revision, current));
        }
        conn.query(&self.source)
    }
}

fn check_session_fresh(conn: &Connection, created: u64) -> Result<()> {
    let current = conn.ddl_revision()?;
    if created != current {
        return Err(session_stale(created, current));
    }
    Ok(())
}

pub(crate) fn session_stale(session_revision: u64, database_revision: u64) -> Error {
    Error::Schema {
        message: format!(
            "VOS-SESSION-STALE: database DDL changed while this session was active \
             (sessionRevision={session_revision}, databaseRevision={database_revision})"
        ),
        span: None,
        hint: Some("close the session, open a new one, and resubmit / re-prepare".into()),
    }
}

pub(crate) fn prepared_stale(prepared_revision: u64, database_revision: u64) -> Error {
    Error::Schema {
        message: format!(
            "VOS-PREPARED-STALE: prepared plan DDL revision mismatch \
             (preparedRevision={prepared_revision}, databaseRevision={database_revision})"
        ),
        span: None,
        hint: Some("re-prepare against the current DDL revision".into()),
    }
}

pub(crate) fn transaction_stale_ddl(txn_revision: u64, database_revision: u64) -> Error {
    Error::Transaction {
        message: format!(
            "VOS-TRANSACTION-STALE-DDL: data transaction crossed a DDL publish \
             (txnRevision={txn_revision}, databaseRevision={database_revision})"
        ),
    }
}
