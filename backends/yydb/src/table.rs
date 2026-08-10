//! Table-row storage validated against a parsed VOS [`vos::ast::Document`].
//!
//! Rows are still persisted in the existing KV snapshot (`records`), keyed as
//! `row/<table>/<pk>`. This is an interim catalog layout until page/B+Tree
//! storage lands — not a parallel schema dialect.

use std::collections::BTreeMap;

use vos::ast::{BuiltinType, Document, Field, Table, TypeExpr};
use yydb_types::{Error, Result, Value};

/// One row of named fields.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// Field values addressed by name (VOS field identity).
    pub fields: BTreeMap<String, Value>,
}

impl Row {
    /// Create an empty row.
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
        }
    }

    /// Insert a field value.
    pub fn insert(&mut self, name: impl Into<String>, value: Value) -> &mut Self {
        self.fields.insert(name.into(), value);
        self
    }

    /// Borrow a field.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

/// Look up a table declaration in a document.
pub fn find_table<'a>(document: &'a Document, name: &str) -> Result<&'a Table> {
    document
        .tables()
        .find(|t| t.name == name)
        .ok_or_else(|| Error::Schema {
            message: format!("unknown table `{name}`"),
            span: None,
            hint: Some("declare the table in the database-truth VOS schema".into()),
        })
}

/// Validate `row` against `table` and return the primary-key value.
pub fn validate_row(table: &Table, row: &Row) -> Result<Value> {
    let primary = table.primary_fields().next().ok_or_else(|| Error::Schema {
        message: format!("table `{}` has no primary key", table.name),
        span: Some((table.span.start, table.span.end)),
        hint: Some("add `@@field: Type` or `[primary] field: Type`".into()),
    })?;

    for field in &table.fields {
        match row.get(&field.name) {
            Some(value) => check_value(&field.ty, value, table, field)?,
            None if type_allows_missing(&field.ty) || field.default.is_some() => {}
            None => {
                return Err(Error::Schema {
                    message: format!("missing field `{}` on table `{}`", field.name, table.name),
                    span: Some((field.span.start, field.span.end)),
                    hint: Some("provide the field or mark it optional / give a default".into()),
                });
            }
        }
    }

    for name in row.fields.keys() {
        if !table.fields.iter().any(|f| f.name == *name) {
            return Err(Error::Schema {
                message: format!("unknown field `{name}` on table `{}`", table.name),
                span: Some((table.span.start, table.span.end)),
                hint: Some("remove the field or add it to the VOS table".into()),
            });
        }
    }

    let pk = row
        .get(&primary.name)
        .cloned()
        .ok_or_else(|| Error::Schema {
            message: format!(
                "primary key `{}` is required on table `{}`",
                primary.name, table.name
            ),
            span: Some((primary.span.start, primary.span.end)),
            hint: None,
        })?;
    if matches!(pk, Value::Null) {
        return Err(Error::Schema {
            message: format!(
                "primary key `{}` on table `{}` must not be null",
                primary.name, table.name
            ),
            span: Some((primary.span.start, primary.span.end)),
            hint: None,
        });
    }
    Ok(pk)
}

/// Ensure `@unique` fields do not collide with other stored rows of `table`.
pub fn check_unique_constraints(
    table: &Table,
    row: &Row,
    pk: &Value,
    existing: &[(Value, Row)],
) -> Result<()> {
    for field in table.fields.iter().filter(|f| f.is_unique()) {
        let Some(value) = row.get(&field.name) else {
            continue;
        };
        if matches!(value, Value::Null) {
            continue;
        }
        for (other_pk, other) in existing {
            if other_pk == pk {
                continue;
            }
            if other.get(&field.name) == Some(value) {
                return Err(Error::Schema {
                    message: format!(
                        "unique constraint failed on `{}.{}`",
                        table.name, field.name
                    ),
                    span: Some((field.span.start, field.span.end)),
                    hint: Some("choose a different value for the unique field".into()),
                });
            }
        }
    }
    Ok(())
}

/// Prefix for all row keys of a table.
pub fn table_row_prefix(table: &str) -> String {
    format!("row/{table}/")
}

fn type_allows_missing(ty: &TypeExpr) -> bool {
    matches!(ty, TypeExpr::Optional(_))
}

fn check_value(ty: &TypeExpr, value: &Value, table: &Table, field: &Field) -> Result<()> {
    match ty {
        TypeExpr::Optional(inner) => {
            if matches!(value, Value::Null) {
                return Ok(());
            }
            check_value(inner, value, table, field)
        }
        TypeExpr::Builtin(builtin) => check_builtin(*builtin, value, table, field),
        TypeExpr::Reference(inner) => {
            // References store the target primary key; accept the inner named/builtin shape.
            match inner.as_ref() {
                TypeExpr::Named(_) | TypeExpr::Builtin(_) => {
                    if matches!(value, Value::Null) {
                        return Err(Error::Schema {
                            message: format!(
                                "field `{}.{}` is a non-optional reference",
                                table.name, field.name
                            ),
                            span: Some((field.span.start, field.span.end)),
                            hint: Some("use `&T?` for optional references or provide a key".into()),
                        });
                    }
                    // PK scalars commonly used today.
                    if matches!(
                        value,
                        Value::Uuid(_) | Value::Text(_) | Value::I64(_) | Value::U64(_)
                    ) {
                        Ok(())
                    } else {
                        Err(type_mismatch(table, field, "reference primary key", value))
                    }
                }
                other => check_value(other, value, table, field),
            }
        }
        TypeExpr::List(_) => Err(Error::Unsupported(
            "list field values in table rows (coming with query/storage layout)",
        )),
        TypeExpr::Named(name) => Err(Error::Schema {
            message: format!(
                "inline named type `{name}` on `{}.{}` is not stored yet",
                table.name, field.name
            ),
            span: Some((field.span.start, field.span.end)),
            hint: Some("use a builtin or `&T` reference for this slice".into()),
        }),
        TypeExpr::Vector { .. } | TypeExpr::File => Err(Error::Unsupported(
            "vector/file row fields (use CAS ObjectRef APIs for now)",
        )),
        _ => Err(Error::Unsupported("unsupported VOS type in table row")),
    }
}

fn check_builtin(builtin: BuiltinType, value: &Value, table: &Table, field: &Field) -> Result<()> {
    let ok = match builtin {
        BuiltinType::Bool => matches!(value, Value::Bool(_)),
        BuiltinType::I8 | BuiltinType::I16 | BuiltinType::I32 | BuiltinType::I64 => {
            matches!(value, Value::I64(_))
        }
        BuiltinType::U8 | BuiltinType::U16 | BuiltinType::U32 | BuiltinType::U64 => {
            matches!(value, Value::U64(_) | Value::I64(_))
        }
        BuiltinType::F32 | BuiltinType::F64 => matches!(value, Value::F64(_)),
        BuiltinType::Utf8 | BuiltinType::Utf16 => matches!(value, Value::Text(_)),
        BuiltinType::Uuid => matches!(value, Value::Uuid(_)),
        BuiltinType::Bytes => matches!(value, Value::Bytes(_)),
        // Until `Value` has a dedicated datetime, accept unix-seconds `i64` or
        // RFC3339-ish `utf8` text produced by hosts / `now()`.
        BuiltinType::DateTimeUtc => matches!(value, Value::I64(_) | Value::Text(_)),
        BuiltinType::Decimal => false,
        _ => false,
    };
    if ok {
        Ok(())
    } else {
        Err(type_mismatch(table, field, builtin.as_vos(), value))
    }
}

fn type_mismatch(table: &Table, field: &Field, expected: &str, value: &Value) -> Error {
    Error::Schema {
        message: format!(
            "type mismatch on `{}.{}`: expected {expected}, got {got}",
            table.name,
            field.name,
            got = value_kind(value)
        ),
        span: Some((field.span.start, field.span.end)),
        hint: None,
    }
}

fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::I64(_) => "i64",
        Value::U64(_) => "u64",
        Value::F64(_) => "f64",
        Value::Bytes(_) => "bytes",
        Value::Text(_) => "utf8",
        Value::Uuid(_) => "uuid",
        Value::Vector(_) => "vector",
        Value::Object(_) => "object",
        Value::File(_) => "file",
        _ => "value",
    }
}

/// Stable KV key for a table primary key.
pub fn row_key(table: &str, pk: &Value) -> Result<String> {
    Ok(format!("row/{table}/{}", pk_token(pk)?))
}

fn pk_token(pk: &Value) -> Result<String> {
    match pk {
        Value::Text(s) => {
            if s.contains('/') || s.contains('\0') {
                return Err(Error::Schema {
                    message: "primary key text must not contain '/' or NUL".into(),
                    span: None,
                    hint: None,
                });
            }
            Ok(format!("t/{s}"))
        }
        Value::Uuid(u) => Ok(format!("u/{}", u.as_hyphenated())),
        Value::I64(n) => Ok(format!("i/{n}")),
        Value::U64(n) => Ok(format!("u64/{n}")),
        _ => Err(Error::Schema {
            message: "unsupported primary key value kind".into(),
            span: None,
            hint: Some("use uuid, utf8, or integer primary keys for this slice".into()),
        }),
    }
}

const TAG_NULL: u8 = 0;
const TAG_BOOL: u8 = 1;
const TAG_I64: u8 = 2;
const TAG_U64: u8 = 3;
const TAG_F64: u8 = 4;
const TAG_TEXT: u8 = 5;
const TAG_UUID: u8 = 6;
const TAG_BYTES: u8 = 7;

/// Encode a row for the KV snapshot.
pub fn encode_row(row: &Row) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    write_u32(&mut out, row.fields.len())?;
    for (name, value) in &row.fields {
        write_bytes(&mut out, name.as_bytes())?;
        encode_value(&mut out, value)?;
    }
    Ok(out)
}

/// Decode a row from the KV snapshot.
pub fn decode_row(bytes: &[u8]) -> Result<Row> {
    let mut cursor = 0;
    let count = read_u32(bytes, &mut cursor)? as usize;
    let mut fields = BTreeMap::new();
    for _ in 0..count {
        let name = String::from_utf8(read_bytes(bytes, &mut cursor)?)
            .map_err(|_| Error::Corrupt("row field name is not UTF-8"))?;
        let value = decode_value(bytes, &mut cursor)?;
        fields.insert(name, value);
    }
    if cursor != bytes.len() {
        return Err(Error::Corrupt("trailing row bytes"));
    }
    Ok(Row { fields })
}

fn encode_value(out: &mut Vec<u8>, value: &Value) -> Result<()> {
    match value {
        Value::Null => out.push(TAG_NULL),
        Value::Bool(v) => {
            out.push(TAG_BOOL);
            out.push(u8::from(*v));
        }
        Value::I64(v) => {
            out.push(TAG_I64);
            out.extend(v.to_le_bytes());
        }
        Value::U64(v) => {
            out.push(TAG_U64);
            out.extend(v.to_le_bytes());
        }
        Value::F64(v) => {
            out.push(TAG_F64);
            out.extend(v.to_le_bytes());
        }
        Value::Text(v) => {
            out.push(TAG_TEXT);
            write_bytes(out, v.as_bytes())?;
        }
        Value::Uuid(v) => {
            out.push(TAG_UUID);
            out.extend(v.as_bytes());
        }
        Value::Bytes(v) => {
            out.push(TAG_BYTES);
            write_bytes(out, v)?;
        }
        _ => {
            return Err(Error::Unsupported(
                "value kind in table row encoding (use scalar fields)",
            ));
        }
    }
    Ok(())
}

fn decode_value(bytes: &[u8], cursor: &mut usize) -> Result<Value> {
    let tag = *bytes
        .get(*cursor)
        .ok_or(Error::Corrupt("truncated row value"))?;
    *cursor += 1;
    match tag {
        TAG_NULL => Ok(Value::Null),
        TAG_BOOL => {
            let b = *bytes.get(*cursor).ok_or(Error::Corrupt("truncated bool"))?;
            *cursor += 1;
            Ok(Value::Bool(b != 0))
        }
        TAG_I64 => Ok(Value::I64(read_i64(bytes, cursor)?)),
        TAG_U64 => Ok(Value::U64(read_u64(bytes, cursor)?)),
        TAG_F64 => Ok(Value::F64(f64::from_le_bytes(take_array(bytes, cursor)?))),
        TAG_TEXT => Ok(Value::Text(
            String::from_utf8(read_bytes(bytes, cursor)?)
                .map_err(|_| Error::Corrupt("text is not UTF-8"))?,
        )),
        TAG_UUID => {
            let raw: [u8; 16] = take_array(bytes, cursor)?;
            Ok(Value::Uuid(uuid::Uuid::from_bytes(raw)))
        }
        TAG_BYTES => Ok(Value::Bytes(read_bytes(bytes, cursor)?)),
        _ => Err(Error::Corrupt("unknown row value tag")),
    }
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
    let raw = take_array::<4>(bytes, cursor)?;
    Ok(u32::from_le_bytes(raw))
}

fn read_i64(bytes: &[u8], cursor: &mut usize) -> Result<i64> {
    Ok(i64::from_le_bytes(take_array(bytes, cursor)?))
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> Result<u64> {
    Ok(u64::from_le_bytes(take_array(bytes, cursor)?))
}

fn read_bytes(bytes: &[u8], cursor: &mut usize) -> Result<Vec<u8>> {
    let len = read_u32(bytes, cursor)? as usize;
    if *cursor + len > bytes.len() {
        return Err(Error::Corrupt("truncated bytes"));
    }
    let out = bytes[*cursor..*cursor + len].to_vec();
    *cursor += len;
    Ok(out)
}

fn take_array<const N: usize>(bytes: &[u8], cursor: &mut usize) -> Result<[u8; N]> {
    if *cursor + N > bytes.len() {
        return Err(Error::Corrupt("truncated fixed bytes"));
    }
    let mut out = [0u8; N];
    out.copy_from_slice(&bytes[*cursor..*cursor + N]);
    *cursor += N;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn roundtrip_row_encoding() {
        let mut row = Row::new();
        row.insert("id", Value::Uuid(Uuid::nil()));
        row.insert("name", Value::Text("a".into()));
        let bytes = encode_row(&row).unwrap();
        assert_eq!(decode_row(&bytes).unwrap(), row);
    }
}
