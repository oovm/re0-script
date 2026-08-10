//! VOS schema documents shared with [`vos`](https://github.com/voml/vos-language)
//! (`git`, branch `dev`).
//!
//! YYDB uses **VOS for DDL and query**. It does **not** invent a private schema
//! dialect. Database-truth documents stored by
//! [`crate::Connection::ensure_schema`] are VOS source text.
//!
//! **Language authority** (grammar, AST, semantic checks, spans, hints) lives in
//! the `vos` crate. Parse/check failures are mapped through
//! [`vos::report_diagnostics`] (miette) so every error keeps its source origin.

use yydb_types::{Error, Result};

/// Canonical remote used by this workspace for shared VOS semantics.
pub const VOS_GIT_DEV: &str = "https://github.com/voml/vos-language.git#branch=dev";

/// Returns true when `document` may be stored without requiring a parsed catalog.
///
/// Empty (or whitespace-only) documents remain allowed so `yydb init` can create
/// a file before the first real schema is written.
pub fn is_empty_document(document: &str) -> bool {
    document.trim().is_empty()
}

/// Parse and check a non-empty VOS document via [`vos::parser`].
pub fn parse_document(document: &str) -> Result<vos::ast::Document> {
    if is_empty_document(document) {
        return Err(Error::Schema {
            message: "expected a VOS schema document".into(),
            span: Some((0, 0)),
            hint: Some("pass a non-empty `.vos` document, or use `yydb init` without `--schema-file` for an empty placeholder".into()),
        });
    }
    vos::parser::parse_document(document).map_err(|diags| map_diagnostics(document, ".vos", diags))
}

/// Validate a schema document before it becomes database truth.
///
/// Non-empty documents are checked by [`vos::parser::parse_document`]. Empty
/// placeholders skip language validation.
pub fn validate_document(document: &str) -> Result<()> {
    if is_empty_document(document) {
        return Ok(());
    }
    parse_document(document).map(|_| ())
}

/// Map VOS diagnostics into [`Error::Vos`] with full source provenance.
pub fn map_diagnostics(source: &str, name: &str, diags: vos::ast::Diagnostics) -> Error {
    Error::Vos(vos::report_diagnostics(source.to_owned(), name, diags))
}

#[cfg(test)]
mod tests {
    use super::*;
    use yydb_types::Error;

    #[test]
    fn accepts_empty_placeholder() {
        validate_document("").unwrap();
        validate_document("   \n").unwrap();
    }

    #[test]
    fn accepts_vos_table() {
        let doc = parse_document(
            r#"
            table Setting {
                @@id: uuid,
                title: utf8,
            }
            "#,
        )
        .unwrap();
        assert!(!doc.items.is_empty());
    }

    #[test]
    fn parse_error_keeps_source_provenance() {
        let source = "table Broken { @@id: uuid\n";
        let err = parse_document(source).unwrap_err();
        match err {
            Error::Vos(report) => {
                assert!(report.source_code().is_some());
                let rendered = format!("{report:?}");
                assert!(
                    rendered.contains("Broken") || report.to_string().contains("expected"),
                    "{report}"
                );
            }
            other => panic!("expected Error::Vos, got {other}"),
        }
    }

    #[test]
    fn rejects_invalid_document() {
        let err = parse_document("not a schema").unwrap_err();
        assert!(err.is_schema_like());
    }
}
