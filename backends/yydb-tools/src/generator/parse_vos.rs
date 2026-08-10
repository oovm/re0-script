//! Interim bridge: [`vos::parser`] document → TypeScript codegen IR.
//!
//! Grammar and semantics live in `vos`. This module only lowers the persistence
//! subset that the MVP emitter understands.

use super::ir::{Field, ScalarType, Table, TsSchemaIr};
use super::Error;

/// Parse VOS through `vos::parser` and lower tables into [`TsSchemaIr`].
pub fn parse_vos_subset(source: &str, schema_version: u32) -> Result<TsSchemaIr, Error> {
    let document = vos::parser::parse_document(source).map_err(map_diags)?;
    let mut tables = Vec::new();
    for table in document.tables() {
        let mut fields = Vec::new();
        for field in &table.fields {
            let ty = scalar_from_vos(&field.ty).ok_or_else(|| {
                Error::Parse(format!(
                    "unsupported field type on `{}.{}` for TypeScript emit (MVP scalars only)",
                    table.name, field.name
                ))
            })?;
            fields.push(Field {
                name: field.name.clone(),
                ty,
                is_id: field.is_primary(),
            });
        }
        tables.push(Table {
            name: table.name.clone(),
            fields,
        });
    }
    if tables.is_empty() {
        return Err(Error::Parse(
            "expected at least one `table` declaration".into(),
        ));
    }
    Ok(TsSchemaIr {
        vos_source: document.source,
        schema_version,
        tables,
    })
}

fn scalar_from_vos(ty: &vos::ast::TypeExpr) -> Option<ScalarType> {
    match ty {
        vos::ast::TypeExpr::Builtin(b) => match b {
            vos::ast::BuiltinType::Uuid => Some(ScalarType::Uuid),
            vos::ast::BuiltinType::Utf8 => Some(ScalarType::Utf8),
            vos::ast::BuiltinType::Bool => Some(ScalarType::Bool),
            vos::ast::BuiltinType::I64 => Some(ScalarType::I64),
            vos::ast::BuiltinType::F64 => Some(ScalarType::F64),
            _ => None,
        },
        _ => None,
    }
}

fn map_diags(diags: vos::ast::Diagnostics) -> Error {
    let message = diags
        .errors
        .first()
        .map(|d| d.to_string())
        .unwrap_or_else(|| "VOS parse error".into());
    Error::Parse(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_setting_table() {
        let ir = parse_vos_subset(
            r#"
            table Setting {
                @@id: uuid,
                key: utf8,
                value: utf8,
            }
            "#,
            1,
        )
        .unwrap();
        assert_eq!(ir.tables.len(), 1);
        assert_eq!(ir.tables[0].name, "Setting");
        assert_eq!(ir.tables[0].fields.len(), 3);
        assert!(ir.tables[0].fields[0].is_id);
        assert_eq!(ir.tables[0].fields[0].name, "id");
    }
}
