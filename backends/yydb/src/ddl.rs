//! DDL session surface (`Database.xxx` host APIs).
//!
//! Ordinary query / data transactions must not mutate the field-identity
//! catalog. [`DdlSession`] is the only write path for rename / add / drop of
//! fields until the VOS `Database.xxx` program executor lands.

use vos::ast::{FieldAttribute, TypeExpr};

use crate::schema_catalog::{FieldId, SchemaCatalog, VirtualFieldIndex};
use crate::Connection;
use yydb_types::{Error, Result};

/// Open DDL write session bound to a [`Connection`].
///
/// Dropping without [`Self::commit`] discards the draft (same as rollback).
pub struct DdlSession<'a> {
    conn: &'a Connection,
}

impl<'a> DdlSession<'a> {
    pub(crate) fn open(conn: &'a Connection) -> Result<Self> {
        conn.open_ddl_draft()?;
        Ok(Self { conn })
    }

    /// Current draft `DdlRevision` (after local mutations, before commit).
    pub fn ddl_revision(&self) -> Result<u64> {
        Ok(self.conn.ddl_draft_catalog()?.revisions.ddl)
    }

    /// `Database.rename_field(Type.old, new)` — identity preserved.
    pub fn rename_field(
        &self,
        type_name: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<(FieldId, VirtualFieldIndex)> {
        self.conn
            .ddl_mutate(|catalog| catalog.rename_field(type_name, old_name, new_name))
    }

    /// `Database.add_field(Type, name, type, …)` — new virtual slot.
    pub fn add_field(
        &self,
        type_name: &str,
        name: &str,
        ty: TypeExpr,
        attrs: Vec<FieldAttribute>,
    ) -> Result<(FieldId, VirtualFieldIndex)> {
        self.conn
            .ddl_mutate(|catalog| catalog.add_field(type_name, name, ty, attrs))
    }

    /// `Database.drop_field(Type.field)` — tombstone; blocked when `dependents` non-empty.
    pub fn drop_field(
        &self,
        type_name: &str,
        field_name: &str,
        dependents: &[&str],
    ) -> Result<VirtualFieldIndex> {
        self.conn
            .ddl_mutate(|catalog| catalog.drop_field(type_name, field_name, dependents))
    }

    /// `Database.move_field` / reorder — changes `SourceOrder` only.
    pub fn reorder_fields(&self, type_name: &str, ordered_names: &[&str]) -> Result<()> {
        self.conn
            .ddl_mutate(|catalog| catalog.reorder_fields(type_name, ordered_names))
    }

    /// Install or replace a durable macro (static field deps; no `.*`).
    pub fn install_macro(
        &self,
        name: &str,
        params: &[(String, TypeExpr)],
        return_ty: TypeExpr,
        body_source: &str,
    ) -> Result<crate::macros::MacroId> {
        self.conn
            .ddl_mutate(|catalog| catalog.install_macro(name, params, return_ty, body_source))
    }

    /// `Database.drop_macro(name)`.
    pub fn drop_macro(&self, name: &str) -> Result<crate::macros::MacroId> {
        self.conn.ddl_mutate(|catalog| catalog.drop_macro(name))
    }

    /// Drop a field, blocked when macros (or other deps) still reference it.
    pub fn drop_field_checked(
        &self,
        type_name: &str,
        field_name: &str,
    ) -> Result<VirtualFieldIndex> {
        self.conn.ddl_mutate(|catalog| {
            let deps = catalog.field_dependents(type_name, field_name)?;
            let listed: Vec<&str> = deps.iter().map(String::as_str).collect();
            catalog.drop_field(type_name, field_name, &listed)
        })
    }

    /// Persist draft catalog + regenerated canonical VOS document atomically.
    pub fn commit(self) -> Result<()> {
        self.conn.commit_ddl_draft()
    }

    /// Discard the draft without writing.
    pub fn rollback(self) {
        self.conn.rollback_ddl_draft();
    }
}

impl Drop for DdlSession<'_> {
    fn drop(&mut self) {
        self.conn.rollback_ddl_draft();
    }
}

/// Ensure a catalog exists for `document`, preferring a durable blob when present.
pub(crate) fn catalog_for_schema(
    document: &str,
    durable: Option<&SchemaCatalog>,
) -> Result<Option<SchemaCatalog>> {
    if crate::schema::is_empty_document(document) {
        return Ok(None);
    }
    if let Some(catalog) = durable {
        return Ok(Some(catalog.clone()));
    }
    let parsed = crate::schema::parse_document(document)?;
    Ok(Some(SchemaCatalog::from_document(&parsed)?))
}

pub(crate) fn require_ddl_session() -> Error {
    Error::Schema {
        message: "VOS-DDL-SESSION-REQUIRED: catalog mutation requires a DDL session".into(),
        span: None,
        hint: Some("call Connection::begin_ddl_session() first".into()),
    }
}
