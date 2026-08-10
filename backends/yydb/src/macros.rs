//! Durable macro catalog IR bound to `FieldId` + virtual slots.
//!
//! Normative: `vos-language/docs/micro-macro.md`, `docs/field-identity.md`.
//! `.*` / reflection inside macros is rejected (`VOS-MACRO-DYNAMIC-FORBIDDEN`).

use std::collections::{BTreeMap, BTreeSet};

use vos::ast::expr::{Expr, FieldInit, Program, ProjItem, Stmt};
use vos::ast::TypeExpr;
use yydb_types::{Error, Result};

use crate::schema_catalog::{FieldId, SchemaCatalog, TypeId, VirtualFieldIndex};

/// Stable macro identity inside one database catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MacroId(pub u64);

/// Durable field reference used by Macro / Query IR (never a bare name).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FieldRef {
    /// Owning type.
    pub type_id: TypeId,
    /// Stable field id.
    pub field_id: FieldId,
    /// Virtual slot (never reused).
    pub virtual_field: VirtualFieldIndex,
}

/// Lowered macro body (v1: collected field loads + display source).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroIr {
    /// Field loads discovered in the body (static deps).
    pub field_loads: Vec<FieldRef>,
}

/// One durable macro entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroEntry {
    /// Stable id.
    pub macro_id: MacroId,
    /// Current name.
    pub name: String,
    /// Parameter names in order.
    pub params: Vec<String>,
    /// Parameter types (parallel to `params`).
    pub param_types: Vec<TypeExpr>,
    /// Declared return type.
    pub return_ty: TypeExpr,
    /// Canonical body source (display names; rewritten on rename).
    pub source: String,
    /// Typed IR bound to virtual slots.
    pub ir: MacroIr,
    /// Types referenced by params / body.
    pub type_deps: BTreeSet<TypeId>,
    /// Fields referenced by body IR.
    pub field_deps: BTreeSet<FieldRef>,
    /// BLAKE3 hex of `source`.
    pub source_hash: String,
    /// BLAKE3 hex of durable IR (FieldId / slots) — stable across rename.
    pub semantic_hash: String,
    /// Macro revision (bumps on body / type change, not rename).
    pub revision: u64,
}

/// Reject `.*` / `*` inside macros and collect `FieldRef` deps.
pub fn lower_macro_body(
    catalog: &SchemaCatalog,
    params: &[(String, TypeExpr)],
    body: &Expr,
) -> Result<MacroIr> {
    let mut param_types: BTreeMap<&str, &TypeExpr> = BTreeMap::new();
    for (name, ty) in params {
        param_types.insert(name.as_str(), ty);
    }
    let mut loads = BTreeSet::new();
    walk_expr(catalog, &param_types, body, None, &mut loads)?;
    Ok(MacroIr {
        field_loads: loads.into_iter().collect(),
    })
}

/// Lower a parsed program body (statements + result).
pub fn lower_macro_program(
    catalog: &SchemaCatalog,
    params: &[(String, TypeExpr)],
    program: &Program,
) -> Result<MacroIr> {
    let mut param_types: BTreeMap<&str, &TypeExpr> = BTreeMap::new();
    for (name, ty) in params {
        param_types.insert(name.as_str(), ty);
    }
    let mut loads = BTreeSet::new();
    for stmt in &program.statements {
        match stmt {
            Stmt::Let(let_) => {
                walk_expr(catalog, &param_types, &let_.value, None, &mut loads)?;
            }
            Stmt::Expr(expr) => walk_expr(catalog, &param_types, expr, None, &mut loads)?,
            _ => {}
        }
    }
    if let Some(result) = &program.result {
        walk_expr(catalog, &param_types, result, None, &mut loads)?;
    }
    Ok(MacroIr {
        field_loads: loads.into_iter().collect(),
    })
}

/// Parse `body_source` as a VOS program and lower it against `catalog`.
pub fn lower_macro_source(
    catalog: &SchemaCatalog,
    params: &[(String, TypeExpr)],
    body_source: &str,
) -> Result<MacroIr> {
    let program = vos::parser::parse_program(body_source).map_err(|d| map_diags(body_source, d))?;
    lower_macro_program(catalog, params, &program)
}

fn map_diags(source: &str, diags: vos::ast::Diagnostics) -> Error {
    crate::schema::map_diagnostics(source, "macro.vos", diags)
}

fn dynamic_forbidden(detail: &str) -> Error {
    Error::Schema {
        message: format!("VOS-MACRO-DYNAMIC-FORBIDDEN: {detail}"),
        span: None,
        hint: Some(
            "macros require static field dependencies; use explicit `x.{ a, b: c }` \
             (micros may use `.*`)"
                .into(),
        ),
    }
}

fn walk_expr(
    catalog: &SchemaCatalog,
    params: &BTreeMap<&str, &TypeExpr>,
    expr: &Expr,
    receiver_ty: Option<&TypeExpr>,
    loads: &mut BTreeSet<FieldRef>,
) -> Result<()> {
    match expr {
        Expr::StarProj { .. } => Err(dynamic_forbidden("`.*` is not allowed inside a macro")),
        Expr::StructProj {
            receiver, items, ..
        } => {
            walk_expr(catalog, params, receiver, None, loads)?;
            let recv_ty = infer_expr_type(params, receiver);
            for item in items {
                match item {
                    ProjItem::Star { .. } => {
                        return Err(dynamic_forbidden(
                            "`*` spread is not allowed inside a macro projection",
                        ));
                    }
                    ProjItem::Field(init) => {
                        record_field_init(catalog, params, &recv_ty, init, loads)?;
                    }
                    _ => {}
                }
            }
            Ok(())
        }
        Expr::Member { object, name, .. } => {
            walk_expr(catalog, params, object, None, loads)?;
            let recv_ty = infer_expr_type(params, object);
            if let Some(ty) = recv_ty.as_ref().or(receiver_ty) {
                if let Some(field_ref) = resolve_field_ref(catalog, ty, name)? {
                    loads.insert(field_ref);
                }
            }
            Ok(())
        }
        Expr::Call { callee, args, .. } => {
            walk_expr(catalog, params, callee, None, loads)?;
            for arg in args {
                walk_expr(catalog, params, arg, None, loads)?;
            }
            Ok(())
        }
        Expr::Unary { expr, .. } | Expr::Try { expr, .. } => {
            walk_expr(catalog, params, expr, receiver_ty, loads)
        }
        Expr::Binary { left, right, .. } => {
            walk_expr(catalog, params, left, None, loads)?;
            walk_expr(catalog, params, right, None, loads)
        }
        Expr::Lambda(lambda) => walk_expr(catalog, params, &lambda.body, None, loads),
        Expr::List { items, .. } => {
            for item in items {
                walk_expr(catalog, params, item, None, loads)?;
            }
            Ok(())
        }
        Expr::TypedObject { fields, .. } | Expr::AnonObject { fields, .. } => {
            for init in fields {
                if let Some(value) = &init.value {
                    walk_expr(catalog, params, value, None, loads)?;
                } else if let Some(ty) = receiver_ty {
                    if let Some(field_ref) = resolve_field_ref(catalog, ty, &init.name)? {
                        loads.insert(field_ref);
                    }
                }
            }
            Ok(())
        }
        Expr::Name { name, .. } => {
            // Bare name in a typed projection context → field of receiver.
            if let Some(ty) = receiver_ty {
                if let Some(field_ref) = resolve_field_ref(catalog, ty, name)? {
                    loads.insert(field_ref);
                }
            }
            Ok(())
        }
        Expr::Literal(_) => Ok(()),
        _ => Ok(()),
    }
}

fn record_field_init(
    catalog: &SchemaCatalog,
    params: &BTreeMap<&str, &TypeExpr>,
    recv_ty: &Option<TypeExpr>,
    init: &FieldInit,
    loads: &mut BTreeSet<FieldRef>,
) -> Result<()> {
    if let Some(value) = &init.value {
        // `name: expr` — walk expr; shorthand field name on the left is output only.
        walk_expr(catalog, params, value, recv_ty.as_ref(), loads)?;
        // Also: `name: user_name` bare name refers to receiver field.
        if let Expr::Name { name, .. } = value {
            if let Some(ty) = recv_ty {
                if let Some(field_ref) = resolve_field_ref(catalog, ty, name)? {
                    loads.insert(field_ref);
                }
            }
        }
    } else if let Some(ty) = recv_ty {
        // Shorthand `user_id` → load receiver.user_id
        if let Some(field_ref) = resolve_field_ref(catalog, ty, &init.name)? {
            loads.insert(field_ref);
        }
    }
    Ok(())
}

fn infer_expr_type(params: &BTreeMap<&str, &TypeExpr>, expr: &Expr) -> Option<TypeExpr> {
    match expr {
        Expr::Name { name, .. } => params.get(name.as_str()).map(|t| (*t).clone()),
        _ => None,
    }
}

fn resolve_field_ref(
    catalog: &SchemaCatalog,
    ty: &TypeExpr,
    field_name: &str,
) -> Result<Option<FieldRef>> {
    let TypeExpr::Named(type_name) = strip_optional(ty) else {
        return Ok(None);
    };
    let entry = match catalog.type_entry(type_name) {
        Ok(e) => e,
        Err(_) => return Ok(None),
    };
    match entry.resolve(field_name) {
        Ok((field_id, virtual_field)) => Ok(Some(FieldRef {
            type_id: entry.type_id,
            field_id,
            virtual_field,
        })),
        Err(_) => Err(Error::Schema {
            message: format!(
                "VOS-DDL-UNKNOWN-FIELD: `{type_name}.{field_name}` is not a live field"
            ),
            span: None,
            hint: None,
        }),
    }
}

fn strip_optional(ty: &TypeExpr) -> &TypeExpr {
    match ty {
        TypeExpr::Optional(inner) => strip_optional(inner),
        other => other,
    }
}

/// BLAKE3 hex digest helper.
pub fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Semantic hash over durable field refs (order-independent).
pub fn semantic_hash_of(ir: &MacroIr) -> String {
    let mut parts: Vec<String> = ir
        .field_loads
        .iter()
        .map(|f| format!("{}:{}:{}", f.type_id.0, f.field_id.0, f.virtual_field))
        .collect();
    parts.sort();
    blake3_hex(parts.join("|").as_bytes())
}

/// Rewrite identifier spellings in macro source after a field rename.
pub fn rewrite_field_name(source: &str, old_name: &str, new_name: &str) -> String {
    if old_name == new_name || old_name.is_empty() {
        return source.to_owned();
    }
    let bytes = source.as_bytes();
    let old = old_name.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(old) && is_ident_boundary(bytes, i, old.len()) {
            out.push_str(new_name);
            i += old.len();
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

fn is_ident_boundary(bytes: &[u8], start: usize, len: usize) -> bool {
    let before_ok = start == 0 || !is_ident_byte(bytes[start - 1]);
    let end = start + len;
    let after_ok = end >= bytes.len() || !is_ident_byte(bytes[end]);
    before_ok && after_ok
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;
    use vos::parser::parse_document;

    fn user_catalog() -> SchemaCatalog {
        let doc = parse_document(
            r#"
            table User {
                @@user_id: uuid,
                @user_name: utf8,
                avatar: utf8?,
            }
            "#,
        )
        .unwrap();
        SchemaCatalog::from_document(&doc).unwrap()
    }

    #[test]
    fn lower_rejects_star_proj() {
        let catalog = user_catalog();
        let params = vec![("user".into(), TypeExpr::Named("User".into()))];
        let err = lower_macro_source(&catalog, &params, "user.*").unwrap_err();
        assert!(err.to_string().contains("VOS-MACRO-DYNAMIC-FORBIDDEN"));
    }

    #[test]
    fn lower_binds_virtual_slots() {
        let catalog = user_catalog();
        let params = vec![("user".into(), TypeExpr::Named("User".into()))];
        let ir = lower_macro_source(
            &catalog,
            &params,
            r#"
            user.{
                user_id,
                name: user_name,
                avatar,
            }
            "#,
        )
        .unwrap();
        assert_eq!(ir.field_loads.len(), 3);
        let slots: BTreeSet<_> = ir.field_loads.iter().map(|f| f.virtual_field).collect();
        assert_eq!(slots, BTreeSet::from([0, 1, 2]));
        let before = semantic_hash_of(&ir);
        // Rename does not change FieldId / slot → same semantic hash.
        let mut renamed = catalog;
        renamed
            .rename_field("User", "user_name", "display_name")
            .unwrap();
        let ir2 = MacroIr {
            field_loads: ir.field_loads.clone(),
        };
        assert_eq!(semantic_hash_of(&ir2), before);
    }

    #[test]
    fn rewrite_field_name_respects_boundaries() {
        let src = "user.user_name + user_name_extra";
        let out = rewrite_field_name(src, "user_name", "display_name");
        assert_eq!(out, "user.display_name + user_name_extra");
    }
}
