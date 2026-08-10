//! Interim secondary-index keys stored beside row payloads in the primary KV map.
//!
//! Unique fields (`@name` / `[unique]`) get `idx/<table>/<field>/<value>` →
//! primary-key token entries. The same UTF-8 key namespace is used on snapshot
//! (`YYDB\x01`) and paged (`YYDB\x02` btree) layouts — no separate index root.
//! Dedicated B+Tree index pages remain a later optimization.

use vos::ast::Table;
use yydb_types::{Error, Result, Value};

use crate::table::{self, Row};

/// Index record key for a unique field value.
pub fn unique_index_key(table: &str, field: &str, value: &Value) -> Result<String> {
    Ok(format!("idx/{table}/{field}/{}", value_token(value)?))
}

fn value_token(value: &Value) -> Result<String> {
    match value {
        Value::Text(s) => {
            if s.contains('/') || s.contains('\0') {
                return Err(Error::Schema {
                    message: "indexed text must not contain '/' or NUL".into(),
                    span: None,
                    hint: None,
                });
            }
            Ok(format!("t/{s}"))
        }
        Value::Uuid(u) => Ok(format!("u/{}", u.as_hyphenated())),
        Value::I64(n) => Ok(format!("i/{n}")),
        Value::U64(n) => Ok(format!("u64/{n}")),
        Value::Bool(b) => Ok(format!("b/{}", u8::from(*b))),
        _ => Err(Error::Schema {
            message: "unsupported indexed value kind".into(),
            span: None,
            hint: Some("index uuid, utf8, bool, or integer fields for this slice".into()),
        }),
    }
}

/// Remove previous unique-index entries that pointed at `pk` for this table.
///
/// Returns the removed index keys (for catalog sidecar mirroring).
pub fn clear_unique_indexes_for_pk(
    records: &mut std::collections::BTreeMap<String, Vec<u8>>,
    table: &str,
    table_def: &Table,
    pk: &Value,
) -> Result<Vec<String>> {
    let pk_bytes = table::row_key(table, pk)?.into_bytes();
    let prefix = format!("idx/{table}/");
    let stale: Vec<String> = records
        .range(prefix.clone()..)
        .take_while(|(k, _)| k.starts_with(&prefix))
        .filter(|(k, v)| {
            table_def
                .fields
                .iter()
                .any(|f| f.is_unique() && k.starts_with(&format!("idx/{table}/{}/", f.name)))
                && **v == pk_bytes
        })
        .map(|(k, _)| k.clone())
        .collect();
    for key in &stale {
        records.remove(key);
    }
    Ok(stale)
}

/// Write unique-index entries for `row` (caller already validated uniqueness).
pub fn write_unique_indexes(
    records: &mut std::collections::BTreeMap<String, Vec<u8>>,
    table: &str,
    table_def: &Table,
    pk: &Value,
    row: &Row,
) -> Result<()> {
    let pk_key = table::row_key(table, pk)?;
    let pk_bytes = pk_key.into_bytes();
    for field in table_def.fields.iter().filter(|f| f.is_unique()) {
        let Some(value) = row.get(&field.name) else {
            continue;
        };
        if matches!(value, Value::Null) {
            continue;
        }
        let key = unique_index_key(table, &field.name, value)?;
        records.insert(key, pk_bytes.clone());
    }
    Ok(())
}

/// Look up an existing primary-key token for a unique field value, if any.
pub fn lookup_unique(
    records: &std::collections::BTreeMap<String, Vec<u8>>,
    table: &str,
    field: &str,
    value: &Value,
) -> Result<Option<Vec<u8>>> {
    let key = unique_index_key(table, field, value)?;
    Ok(records.get(&key).cloned())
}
