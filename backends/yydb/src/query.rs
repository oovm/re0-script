//! VOS operation-language: parse → lower → execute (object / method style).
//!
//! Normative design:
//!
//! - `vos-language/docs/expressions.md`
//! - `vos-language/docs/operations.md`
//!
//! v1 executor covers: `Type.get`, `Type {…}.insert()`, filter / map / sort_by /
//! skip / take / collect, entity/collection update/delete, and immutable `let`
//! bindings.
//!
//! **Indexed reads:** a leading `.filter(x => x.<pk|unique> == const)` (also
//! via `.first` / `.any` / `.count`) uses primary-key or unique-index lookup
//! instead of a full table scan. Compound predicates and non-equality filters
//! still scan.

use std::collections::BTreeMap;

use vos::ast::codes;
use vos::ast::expr::{BinaryOp, Expr, FieldInit, Lambda, Program, ProjItem, Stmt, UnaryOp};
use vos::ast::op::{QueryPlan, SortDir, SortKey, Stage, TableRef};
use vos::ast::{Literal, Span};
use yydb_types::{Error, Result, Value};

use crate::table::{self, Row};
use crate::Connection;

/// Map parser diagnostics into [`Error::Vos`] (miette + source).
pub fn map_program_diagnostics(source: &str, diags: vos::ast::Diagnostics) -> Error {
    crate::schema::map_diagnostics(source, "query.vos", diags)
}

/// Parse and execute a VOS object-ops program; return rows from the result expr.
pub fn execute_program(conn: &Connection, source: &str) -> Result<Vec<Row>> {
    let program =
        vos::parser::parse_program(source).map_err(|d| map_program_diagnostics(source, d))?;
    run_program(conn, &program)
}

/// Execute an already-parsed VOS program (used by [`crate::PreparedPlan`]).
pub(crate) fn run_program(conn: &Connection, program: &Program) -> Result<Vec<Row>> {
    let mut env: BTreeMap<String, Rt> = BTreeMap::new();
    for stmt in &program.statements {
        match stmt {
            Stmt::Let(let_) => {
                let value = eval_expr(conn, &let_.value, &env, None)?;
                env.insert(let_.name.clone(), value);
            }
            Stmt::Expr(expr) => {
                let _ = eval_expr(conn, expr, &env, None)?;
            }
            _ => {
                return Err(Error::Unsupported("VOS statement kind"));
            }
        }
    }
    match &program.result {
        Some(expr) => rt_to_rows(eval_expr(conn, expr, &env, None)?),
        None => {
            // Last let binding if any.
            if let Some(Stmt::Let(let_)) = program.statements.last() {
                if let Some(v) = env.get(&let_.name) {
                    return rt_to_rows(v.clone());
                }
            }
            Ok(Vec::new())
        }
    }
}

#[derive(Debug, Clone)]
enum Rt {
    Null,
    Value(Value),
    Row(Row),
    Rows(Vec<Row>),
    Plan(QueryPlan),
    Draft { table: String, row: Row },
    Drafts { table: String, rows: Vec<Row> },
}

fn rt_to_rows(rt: Rt) -> Result<Vec<Row>> {
    match rt {
        Rt::Rows(rows) => Ok(rows),
        Rt::Row(row) => Ok(vec![row]),
        Rt::Null => Ok(Vec::new()),
        Rt::Plan(_) => Err(Error::Schema {
            message: format!(
                "{}: lazy query must end with an execution boundary (e.g. `.collect()`)",
                codes::OP_0005
            ),
            span: None,
            hint: Some("call `.collect()`, `.first()`, `.get()`, or a write method".into()),
        }),
        Rt::Draft { .. } | Rt::Drafts { .. } => Err(Error::Schema {
            message: "typed construction is not a query result; call `.insert()` to persist".into(),
            span: None,
            hint: None,
        }),
        Rt::Value(Value::Null) => Ok(Vec::new()),
        Rt::Value(other) => {
            let mut row = Row::new();
            row.insert("value", other);
            Ok(vec![row])
        }
    }
}

fn eval_expr(
    conn: &Connection,
    expr: &Expr,
    env: &BTreeMap<String, Rt>,
    row_ctx: Option<&Row>,
) -> Result<Rt> {
    match expr {
        Expr::Literal(lit) => Ok(Rt::Value(literal_to_value(lit)?)),
        Expr::Name { name, .. } => {
            if let Some(rt) = env.get(name) {
                return Ok(rt.clone());
            }
            if let Some(row) = row_ctx {
                if let Some(v) = row.get(name) {
                    return Ok(Rt::Value(v.clone()));
                }
            }
            // Bare table name ? empty plan (collection entry).
            if conn
                .parsed_schema()?
                .map(|doc| doc.tables().any(|t| t.name == *name))
                .unwrap_or(false)
            {
                return Ok(Rt::Plan(QueryPlan::all(TableRef {
                    name: name.clone(),
                    span: Span::empty(0),
                })));
            }
            Err(Error::Schema {
                message: format!("{}: unknown name `{name}`", codes::EXPR_0001),
                span: None,
                hint: None,
            })
        }
        Expr::TypedObject { ty, fields, .. } => {
            let row = eval_field_inits(conn, fields, env, row_ctx)?;
            Ok(Rt::Draft {
                table: ty.clone(),
                row,
            })
        }
        Expr::AnonObject { fields, .. } => {
            let row = eval_field_inits(conn, fields, env, row_ctx)?;
            Ok(Rt::Row(row))
        }
        Expr::List { items, .. } => {
            let mut table: Option<String> = None;
            let mut rows = Vec::new();
            for item in items {
                match eval_expr(conn, item, env, row_ctx)? {
                    Rt::Draft { table: t, row } => {
                        if let Some(existing) = &table {
                            if existing != &t {
                                return Err(Error::Schema {
                                    message: "list.insert requires a homogeneous typed list".into(),
                                    span: None,
                                    hint: None,
                                });
                            }
                        } else {
                            table = Some(t);
                        }
                        rows.push(row);
                    }
                    Rt::Row(row) => rows.push(row),
                    other => {
                        return Err(Error::Unsupported(match other {
                            Rt::Value(_) => "list of scalars in object-ops",
                            _ => "list element kind in object-ops",
                        }));
                    }
                }
            }
            if let Some(table) = table {
                Ok(Rt::Drafts { table, rows })
            } else {
                Ok(Rt::Rows(rows))
            }
        }
        Expr::Unary {
            op: UnaryOp::Not,
            expr,
            ..
        } => {
            let v = eval_expr(conn, expr, env, row_ctx)?;
            Ok(Rt::Value(Value::Bool(!rt_truthy(&v)?)))
        }
        Expr::Binary {
            op, left, right, ..
        } => eval_binary(conn, *op, left, right, env, row_ctx),
        Expr::Member { object, name, .. } => {
            let recv = eval_expr(conn, object, env, row_ctx)?;
            match recv {
                Rt::Row(row) | Rt::Draft { row, .. } => match row.get(name) {
                    Some(v) => Ok(Rt::Value(v.clone())),
                    None => Ok(Rt::Value(Value::Null)),
                },
                Rt::Plan(plan) if name == "all" => {
                    // Member `.all` without call ? treat as stage when called.
                    Ok(Rt::Plan(plan))
                }
                _ => Err(Error::Schema {
                    message: format!("cannot access field `{name}` on this value"),
                    span: None,
                    hint: None,
                }),
            }
        }
        Expr::Call { callee, args, span } => eval_call(conn, callee, args, *span, env, row_ctx),
        Expr::Lambda(_) => Err(Error::Schema {
            message: "lambda cannot be evaluated outside a method argument".into(),
            span: None,
            hint: None,
        }),
        Expr::StarProj { receiver, .. } => {
            let recv = eval_expr(conn, receiver, env, row_ctx)?;
            match recv {
                Rt::Row(row) | Rt::Draft { row, .. } => Ok(Rt::Row(row)),
                _ => Err(Error::Unsupported("`.*` on non-row")),
            }
        }
        Expr::StructProj {
            receiver, items, ..
        } => {
            let recv = eval_expr(conn, receiver, env, row_ctx)?;
            let base = match recv {
                Rt::Row(row) | Rt::Draft { row, .. } => row,
                _ => {
                    return Err(Error::Unsupported("projection receiver must be a row"));
                }
            };
            Ok(Rt::Row(apply_projection(conn, &base, items, env)?))
        }
        Expr::Try { expr, .. } => {
            let v = eval_expr(conn, expr, env, row_ctx)?;
            match &v {
                Rt::Null | Rt::Value(Value::Null) => {
                    return Err(Error::Schema {
                        message: "optional value was null".into(),
                        span: None,
                        hint: Some("handle absence or ensure the row exists".into()),
                    });
                }
                Rt::Rows(rows) if rows.is_empty() => {
                    return Err(Error::Schema {
                        message: "optional value was null".into(),
                        span: None,
                        hint: Some("handle absence or ensure the row exists".into()),
                    });
                }
                _ => {}
            }
            Ok(v)
        }
        _ => Err(Error::Unsupported("VOS expression kind")),
    }
}

fn eval_field_inits(
    conn: &Connection,
    fields: &[FieldInit],
    env: &BTreeMap<String, Rt>,
    row_ctx: Option<&Row>,
) -> Result<Row> {
    let mut row = Row::new();
    for field in fields {
        let value = match &field.value {
            Some(expr) => match eval_expr(conn, expr, env, row_ctx)? {
                Rt::Value(v) => v,
                Rt::Null => Value::Null,
                Rt::Row(_) | Rt::Draft { .. } | Rt::Drafts { .. } => {
                    return Err(Error::Unsupported("nested row field in constructor"));
                }
                other => {
                    return Err(Error::Unsupported(match other {
                        Rt::Rows(_) => "rows in field init",
                        Rt::Plan(_) => "plan in field init",
                        _ => "field init value",
                    }));
                }
            },
            None => {
                // Shorthand: look up name in env / row context.
                match eval_expr(conn, &Expr::name(&field.name, Span::empty(0)), env, row_ctx)? {
                    Rt::Value(v) => v,
                    Rt::Null => Value::Null,
                    _ => {
                        return Err(Error::Schema {
                            message: format!("cannot resolve shorthand field `{}`", field.name),
                            span: Some((field.span.start, field.span.end)),
                            hint: None,
                        });
                    }
                }
            }
        };
        row.insert(field.name.clone(), value);
    }
    Ok(row)
}

fn apply_projection(
    conn: &Connection,
    base: &Row,
    items: &[ProjItem],
    env: &BTreeMap<String, Rt>,
) -> Result<Row> {
    let mut out = Row::new();
    for item in items {
        match item {
            ProjItem::Star { .. } => {
                for (k, v) in &base.fields {
                    if out.fields.contains_key(k) {
                        return Err(Error::Schema {
                            message: format!(
                                "{}: projection defines `{k}` more than once",
                                codes::PROJECTION_0004
                            ),
                            span: None,
                            hint: Some("use a different output field name".into()),
                        });
                    }
                    out.insert(k.clone(), v.clone());
                }
            }
            ProjItem::Field(init) => {
                if out.fields.contains_key(&init.name) {
                    return Err(Error::Schema {
                        message: format!(
                            "{}: projection defines `{}` more than once",
                            codes::PROJECTION_0001,
                            init.name
                        ),
                        span: Some((init.span.start, init.span.end)),
                        hint: None,
                    });
                }
                let value = match &init.value {
                    None => base.get(&init.name).cloned().unwrap_or(Value::Null),
                    Some(expr) => match eval_expr(conn, expr, env, Some(base))? {
                        Rt::Value(v) => v,
                        Rt::Null => Value::Null,
                        _ => {
                            return Err(Error::Unsupported("projection expression result"));
                        }
                    },
                };
                out.insert(init.name.clone(), value);
            }
            _ => {
                return Err(Error::Unsupported("projection item"));
            }
        }
    }
    Ok(out)
}

fn eval_call(
    conn: &Connection,
    callee: &Expr,
    args: &[Expr],
    span: Span,
    env: &BTreeMap<String, Rt>,
    row_ctx: Option<&Row>,
) -> Result<Rt> {
    // Builtin free calls: `uuid()`, `now()`.
    if let Expr::Name { name, .. } = callee {
        if name == "uuid" && args.is_empty() {
            return Ok(Rt::Value(Value::Uuid(uuid::Uuid::new_v4())));
        }
        if name == "now" && args.is_empty() {
            // Unix seconds as i64 until Value gains a dedicated DateTime variant.
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            return Ok(Rt::Value(Value::I64(secs)));
        }
    }

    // Method call: recv.method(args) encoded as Call { callee: Member { object, name }, args }
    if let Expr::Member {
        object,
        name: method,
        ..
    } = callee
    {
        let recv = eval_expr(conn, object, env, row_ctx)?;
        return dispatch_method(conn, recv, method, args, span, env, row_ctx);
    }

    Err(Error::Schema {
        message: format!("{}: unsupported call", codes::OP_0002),
        span: Some((span.start, span.end)),
        hint: None,
    })
}

fn dispatch_method(
    conn: &Connection,
    recv: Rt,
    method: &str,
    args: &[Expr],
    span: Span,
    env: &BTreeMap<String, Rt>,
    row_ctx: Option<&Row>,
) -> Result<Rt> {
    match (recv, method) {
        (Rt::Plan(plan), "filter") => {
            let pred = expect_one_arg(args, "filter")?;
            Ok(Rt::Plan(plan.filter(pred.clone(), span)))
        }
        (Rt::Plan(plan), "map") => {
            let proj = expect_one_arg(args, "map")?;
            Ok(Rt::Plan(plan.map(proj.clone(), span)))
        }
        (Rt::Plan(plan), "sort_by") => {
            let key = expect_one_arg(args, "sort_by")?;
            let mut plan = plan;
            plan.stages.push(Stage::Sort {
                keys: vec![SortKey {
                    expr: key.clone(),
                    dir: SortDir::Asc,
                    span,
                }],
                span,
            });
            Ok(Rt::Plan(plan))
        }
        (Rt::Plan(plan), "sort_by_desc") => {
            let key = expect_one_arg(args, "sort_by_desc")?;
            let mut plan = plan;
            plan.stages.push(Stage::Sort {
                keys: vec![SortKey {
                    expr: key.clone(),
                    dir: SortDir::Desc,
                    span,
                }],
                span,
            });
            Ok(Rt::Plan(plan))
        }
        (Rt::Plan(plan), "skip") => {
            let count = expect_one_arg(args, "skip")?;
            let mut plan = plan;
            plan.stages.push(Stage::Skip {
                count: count.clone(),
                span,
            });
            Ok(Rt::Plan(plan))
        }
        (Rt::Plan(plan), "take") => {
            let count = expect_one_arg(args, "take")?;
            let mut plan = plan;
            plan.stages.push(Stage::Take {
                count: count.clone(),
                span,
            });
            Ok(Rt::Plan(plan))
        }
        (Rt::Plan(plan), "all") => {
            if !args.is_empty() {
                return Err(Error::Schema {
                    message: "`.all()` takes no arguments".into(),
                    span: Some((span.start, span.end)),
                    hint: None,
                });
            }
            Ok(Rt::Plan(plan))
        }
        (Rt::Plan(plan), "collect") => {
            if !args.is_empty() {
                return Err(Error::Schema {
                    message: "`.collect()` takes no arguments".into(),
                    span: Some((span.start, span.end)),
                    hint: None,
                });
            }
            Ok(Rt::Rows(execute_plan(conn, &plan, env)?))
        }
        (Rt::Plan(plan), "get") => {
            let key_expr = expect_one_arg(args, "get")?;
            let key = match eval_expr(conn, key_expr, env, row_ctx)? {
                Rt::Value(v) => coerce_pk_value(conn, &plan.source.name, v)?,
                _ => {
                    return Err(Error::Schema {
                        message: "`.get` expects a primary-key value".into(),
                        span: Some((span.start, span.end)),
                        hint: None,
                    });
                }
            };
            match conn.get_row(&plan.source.name, &key)? {
                Some(row) => Ok(Rt::Row(row)),
                None => Ok(Rt::Null),
            }
        }
        (Rt::Plan(plan), "first") => {
            let pred = expect_one_arg(args, "first")?;
            let mut filtered = plan.filter(pred.clone(), span);
            // take 1 after filter
            filtered.stages.push(Stage::Take {
                count: Expr::Literal(Literal::Int("1".into())),
                span,
            });
            let rows = execute_plan(conn, &filtered, env)?;
            Ok(rows.into_iter().next().map(Rt::Row).unwrap_or(Rt::Null))
        }
        (Rt::Plan(plan), "count") => {
            let plan = if args.is_empty() {
                plan
            } else {
                plan.filter(expect_one_arg(args, "count")?.clone(), span)
            };
            let n = execute_plan(conn, &plan, env)?.len() as i64;
            Ok(Rt::Value(Value::I64(n)))
        }
        (Rt::Plan(plan), "any") => {
            let pred = expect_one_arg(args, "any")?;
            let filtered = plan.filter(pred.clone(), span);
            let rows = execute_plan(conn, &filtered, env)?;
            Ok(Rt::Value(Value::Bool(!rows.is_empty())))
        }
        (Rt::Plan(plan), "delete") => {
            if !args.is_empty() {
                return Err(Error::Schema {
                    message: "`.delete()` takes no arguments".into(),
                    span: Some((span.start, span.end)),
                    hint: None,
                });
            }
            let rows = execute_plan(conn, &plan, env)?;
            let table = &plan.source.name;
            let document = conn.parsed_schema()?.ok_or_else(|| Error::Schema {
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
            let mut affected = 0u64;
            for row in &rows {
                if let Some(pk) = row.get(pk_name) {
                    if conn.delete_row(table, pk)? {
                        affected += 1;
                    }
                }
            }
            Ok(Rt::Value(Value::U64(affected)))
        }
        (Rt::Plan(plan), "update") => {
            let patch_expr = expect_one_arg(args, "update")?;
            let rows = execute_plan(conn, &plan, env)?;
            let table = plan.source.name.clone();
            let mut out = Vec::new();
            for row in rows {
                let patched = apply_patch(conn, &row, patch_expr, env)?;
                conn.put_row(&table, patched.clone())?;
                out.push(patched);
            }
            Ok(Rt::Rows(out))
        }
        (Rt::Draft { table, row }, "insert") => {
            if !args.is_empty() {
                return Err(Error::Schema {
                    message: "`.insert()` takes no arguments".into(),
                    span: Some((span.start, span.end)),
                    hint: None,
                });
            }
            conn.put_row(&table, row.clone())?;
            Ok(Rt::Row(row))
        }
        (Rt::Drafts { table, rows }, "insert") => {
            if !args.is_empty() {
                return Err(Error::Schema {
                    message: "`.insert()` takes no arguments".into(),
                    span: Some((span.start, span.end)),
                    hint: None,
                });
            }
            let mut out = Vec::new();
            for row in rows {
                conn.put_row(&table, row.clone())?;
                out.push(row);
            }
            Ok(Rt::Rows(out))
        }
        (Rt::Rows(_), "insert") => Err(Error::Unsupported(
            "list.insert() requires typed `Type { ? }` elements",
        )),
        (Rt::Row(row), "update") => {
            let patch_expr = expect_one_arg(args, "update")?;
            let table = find_table_for_row(conn, &row)?;
            let patched = apply_patch(conn, &row, patch_expr, env)?;
            conn.put_row(&table, patched.clone())?;
            Ok(Rt::Row(patched))
        }
        (Rt::Row(row), "delete") => {
            let table = find_table_for_row(conn, &row)?;
            let document = conn.parsed_schema()?.unwrap();
            let table_def = table::find_table(&document, &table)?;
            let pk_name = table_def.primary_fields().next().unwrap().name.as_str();
            let pk = row.get(pk_name).cloned().ok_or_else(|| Error::Schema {
                message: "row missing primary key".into(),
                span: None,
                hint: None,
            })?;
            let _ = conn.delete_row(&table, &pk)?;
            Ok(Rt::Value(Value::U64(1)))
        }
        (Rt::Value(Value::Text(s)), "lower") if args.is_empty() => {
            Ok(Rt::Value(Value::Text(s.to_lowercase())))
        }
        (Rt::Value(Value::Text(s)), "trim") if args.is_empty() => {
            Ok(Rt::Value(Value::Text(s.trim().to_owned())))
        }
        (Rt::Value(Value::Text(s)), "starts_with") => {
            let arg = expect_one_arg(args, "starts_with")?;
            let prefix = match eval_expr(conn, arg, env, row_ctx)? {
                Rt::Value(Value::Text(p)) => p,
                _ => {
                    return Err(Error::Schema {
                        message: "starts_with expects a string".into(),
                        span: None,
                        hint: None,
                    });
                }
            };
            Ok(Rt::Value(Value::Bool(s.starts_with(&prefix))))
        }
        (Rt::Value(Value::Text(s)), "contains") => {
            let arg = expect_one_arg(args, "contains")?;
            let needle = match eval_expr(conn, arg, env, row_ctx)? {
                Rt::Value(Value::Text(p)) => p,
                _ => {
                    return Err(Error::Schema {
                        message: "contains expects a string".into(),
                        span: None,
                        hint: None,
                    });
                }
            };
            Ok(Rt::Value(Value::Bool(s.contains(&needle))))
        }
        (_, method) => Err(Error::Schema {
            message: format!(
                "{}: method `{method}` is not valid on this receiver",
                codes::OP_0002
            ),
            span: Some((span.start, span.end)),
            hint: None,
        }),
    }
}

fn find_table_for_row(conn: &Connection, row: &Row) -> Result<String> {
    let document = conn.parsed_schema()?.ok_or_else(|| Error::Schema {
        message: "database has no VOS schema".into(),
        span: None,
        hint: None,
    })?;
    for table in document.tables() {
        let Some(pk) = table.primary_fields().next() else {
            continue;
        };
        let Some(pk_val) = row.get(&pk.name) else {
            continue;
        };
        if conn.get_row(&table.name, pk_val)?.is_some() {
            return Ok(table.name.clone());
        }
    }
    Err(Error::Schema {
        message: "cannot resolve table for entity update/delete".into(),
        span: None,
        hint: Some("use `Type.filter(?).update(?)` or keep the row from `Type.get`".into()),
    })
}

fn apply_patch(
    conn: &Connection,
    row: &Row,
    patch_expr: &Expr,
    env: &BTreeMap<String, Rt>,
) -> Result<Row> {
    let fields = match patch_expr {
        Expr::AnonObject { fields, .. } => fields.as_slice(),
        Expr::Lambda(Lambda { body, .. }) => match body.as_ref() {
            Expr::AnonObject { fields, .. } => fields.as_slice(),
            _ => {
                return Err(Error::Schema {
                    message: format!(
                        "{}: `.update` expects a patch object `{{ field: expr }}`",
                        codes::OP_0004
                    ),
                    span: None,
                    hint: None,
                });
            }
        },
        Expr::StructProj { .. } => {
            return Err(Error::Schema {
                message: format!(
                    "{}: `.update` argument is a projection, not a patch",
                    codes::OP_0004
                ),
                span: None,
                hint: Some("use `{ field: expr }` not `x.{{ ? }}`".into()),
            });
        }
        _ => {
            return Err(Error::Schema {
                message: format!("{}: `.update` expects a patch object", codes::OP_0004),
                span: None,
                hint: None,
            });
        }
    };
    let mut out = row.clone();
    let patch_row = eval_field_inits(conn, fields, env, Some(row))?;
    for (k, v) in patch_row.fields {
        out.insert(k, v);
    }
    Ok(out)
}

fn expect_one_arg<'a>(args: &'a [Expr], method: &str) -> Result<&'a Expr> {
    if args.len() != 1 {
        return Err(Error::Schema {
            message: format!("`.{method}` expects exactly one argument"),
            span: None,
            hint: None,
        });
    }
    Ok(&args[0])
}

fn coerce_pk_value(conn: &Connection, table: &str, value: Value) -> Result<Value> {
    let document = conn.parsed_schema()?.ok_or_else(|| Error::Schema {
        message: "database has no VOS schema".into(),
        span: None,
        hint: None,
    })?;
    let table_def = table::find_table(&document, table)?;
    let Some(pk) = table_def.primary_fields().next() else {
        return Ok(value);
    };
    coerce_field_value(table, pk, value)
}

fn coerce_field_value(table: &str, field: &vos::ast::Field, value: Value) -> Result<Value> {
    match (&field.ty, value) {
        (vos::ast::TypeExpr::Builtin(vos::ast::BuiltinType::Uuid), Value::Text(text)) => {
            let uuid = uuid::Uuid::parse_str(&text).map_err(|_| Error::Schema {
                message: format!("invalid uuid literal `{text}` for `{table}.{}`", field.name),
                span: Some((field.span.start, field.span.end)),
                hint: None,
            })?;
            Ok(Value::Uuid(uuid))
        }
        (_, other) => Ok(other),
    }
}

fn execute_plan(
    conn: &Connection,
    plan: &QueryPlan,
    env: &BTreeMap<String, Rt>,
) -> Result<Vec<Row>> {
    let (mut rows, stage_start) = seed_rows_for_plan(conn, plan, env)?;

    for stage in plan.stages.iter().skip(stage_start) {
        match stage {
            Stage::All { .. } => {}
            Stage::Filter { predicate, .. } => {
                let mut next = Vec::new();
                for row in rows {
                    if eval_predicate(conn, predicate, &row, env)? {
                        next.push(row);
                    }
                }
                rows = next;
            }
            Stage::Map { projection, .. } => {
                let mut next = Vec::new();
                for row in &rows {
                    next.push(eval_map(conn, projection, row, env)?);
                }
                rows = next;
            }
            Stage::Sort { keys, .. } => {
                rows.sort_by(|a, b| {
                    for key in keys {
                        let av = eval_sort_key(conn, &key.expr, a, env).ok();
                        let bv = eval_sort_key(conn, &key.expr, b, env).ok();
                        let ord = cmp_opt_value(av.as_ref(), bv.as_ref());
                        let ord = match key.dir {
                            SortDir::Asc => ord,
                            SortDir::Desc => ord.reverse(),
                            _ => ord,
                        };
                        if ord != core::cmp::Ordering::Equal {
                            return ord;
                        }
                    }
                    core::cmp::Ordering::Equal
                });
            }
            Stage::Skip { count, .. } => {
                let n = eval_usize(conn, count, env)?;
                if n >= rows.len() {
                    rows.clear();
                } else {
                    rows = rows.split_off(n);
                }
            }
            Stage::Take { count, .. } => {
                let n = eval_usize(conn, count, env)?;
                if rows.len() > n {
                    rows.truncate(n);
                }
            }
            Stage::Load { .. } => {
                return Err(Error::Unsupported(".load association"));
            }
            _ => {
                return Err(Error::Unsupported("query stage"));
            }
        }
    }
    Ok(rows)
}

/// Seed plan rows: full scan, or PK / unique-index lookup when the first
/// non-`All` stage is a simple `field == const` on an indexed field.
///
/// Returns `(rows, stage_index)` where stages before `stage_index` are done
/// (`All` skipped; equality Filter consumed when lookup applied).
fn seed_rows_for_plan(
    conn: &Connection,
    plan: &QueryPlan,
    env: &BTreeMap<String, Rt>,
) -> Result<(Vec<Row>, usize)> {
    let mut i = 0usize;
    while matches!(plan.stages.get(i), Some(Stage::All { .. })) {
        i += 1;
    }

    if let Some(Stage::Filter { predicate, .. }) = plan.stages.get(i) {
        if let Some((field_name, value_expr)) = match_eq_field_const(predicate) {
            if let Some(looked_up) =
                try_indexed_eq_lookup(conn, &plan.source.name, &field_name, value_expr, env)?
            {
                let rows = looked_up.into_iter().collect();
                return Ok((rows, i + 1));
            }
        }
    }

    let rows: Vec<Row> = conn
        .scan_table_rows(&plan.source.name)?
        .into_iter()
        .map(|(_, r)| r)
        .collect();
    Ok((rows, 0))
}

/// `x => x.field == const` / `x => const == x.field` (const may use env).
fn match_eq_field_const(predicate: &Expr) -> Option<(String, &Expr)> {
    let (params, body) = match predicate {
        Expr::Lambda(Lambda { params, body, .. }) if params.len() == 1 => {
            (Some(params[0].as_str()), body.as_ref())
        }
        other => (None, other),
    };
    let Expr::Binary {
        op: BinaryOp::Eq,
        left,
        right,
        ..
    } = body
    else {
        return None;
    };
    if let Some(field) = match_param_field(left, params) {
        return Some((field, right.as_ref()));
    }
    if let Some(field) = match_param_field(right, params) {
        return Some((field, left.as_ref()));
    }
    None
}

fn match_param_field(expr: &Expr, param: Option<&str>) -> Option<String> {
    match expr {
        Expr::Member {
            object: box_obj,
            name,
            ..
        } => match box_obj.as_ref() {
            Expr::Name { name: recv, .. } => {
                if param.map(|p| p == recv).unwrap_or(true) {
                    Some(name.clone())
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => None,
    }
}

/// Look up by primary key or unique index. `None` = field is not indexed /
/// lookup path not applicable (caller should scan).
fn try_indexed_eq_lookup(
    conn: &Connection,
    table: &str,
    field_name: &str,
    value_expr: &Expr,
    env: &BTreeMap<String, Rt>,
) -> Result<Option<Option<Row>>> {
    let document = conn.parsed_schema()?.ok_or_else(|| Error::Schema {
        message: "database has no VOS schema".into(),
        span: None,
        hint: None,
    })?;
    let table_def = table::find_table(&document, table)?;
    let Some(field) = table_def.fields.iter().find(|f| f.name == field_name) else {
        return Ok(None);
    };
    if !field.is_primary() && !field.is_unique() {
        return Ok(None);
    }

    let raw = match eval_expr(conn, value_expr, env, None)? {
        Rt::Value(v) => v,
        Rt::Null => Value::Null,
        _ => {
            return Err(Error::Schema {
                message: format!("indexed equality on `{table}.{field_name}` expects a scalar"),
                span: None,
                hint: None,
            });
        }
    };
    if matches!(raw, Value::Null) {
        // Null never matches unique/PK index entries in this storage model.
        return Ok(Some(None));
    }
    let value = coerce_field_value(table, field, raw)?;

    if field.is_primary() {
        return Ok(Some(conn.get_row(table, &value)?));
    }
    Ok(Some(conn.get_row_by_unique(table, field_name, &value)?))
}

fn eval_predicate(
    conn: &Connection,
    predicate: &Expr,
    row: &Row,
    env: &BTreeMap<String, Rt>,
) -> Result<bool> {
    let body = match predicate {
        Expr::Lambda(Lambda { params, body, .. }) => {
            if params.len() != 1 {
                return Err(Error::Schema {
                    message: "filter lambda must have one parameter".into(),
                    span: None,
                    hint: None,
                });
            }
            // Bind param name to row via row_ctx; also allow x.field through Member on Name(param)
            body.as_ref()
        }
        other => other,
    };
    // For `x.field`, Member evaluates object Name ? need param as row.
    // Inject param into a local env as Row.
    let mut local = env.clone();
    if let Expr::Lambda(Lambda { params, .. }) = predicate {
        local.insert(params[0].clone(), Rt::Row(row.clone()));
    }
    let v = eval_expr(conn, body, &local, Some(row))?;
    rt_truthy(&v)
}

fn eval_map(
    conn: &Connection,
    projection: &Expr,
    row: &Row,
    env: &BTreeMap<String, Rt>,
) -> Result<Row> {
    let mut local = env.clone();
    let body = match projection {
        Expr::Lambda(Lambda { params, body, .. }) => {
            if params.len() != 1 {
                return Err(Error::Schema {
                    message: "map lambda must have one parameter".into(),
                    span: None,
                    hint: None,
                });
            }
            local.insert(params[0].clone(), Rt::Row(row.clone()));
            body.as_ref()
        }
        other => other,
    };
    match eval_expr(conn, body, &local, Some(row))? {
        Rt::Row(r) => Ok(r),
        Rt::Draft { row, .. } => Ok(row),
        _ => Err(Error::Schema {
            message: "map must produce a structural row (`x.*` or `x.{ ? }`)".into(),
            span: None,
            hint: None,
        }),
    }
}

fn eval_sort_key(
    conn: &Connection,
    key: &Expr,
    row: &Row,
    env: &BTreeMap<String, Rt>,
) -> Result<Value> {
    let mut local = env.clone();
    let body = match key {
        Expr::Lambda(Lambda { params, body, .. }) => {
            if !params.is_empty() {
                local.insert(params[0].clone(), Rt::Row(row.clone()));
            }
            body.as_ref()
        }
        other => other,
    };
    match eval_expr(conn, body, &local, Some(row))? {
        Rt::Value(v) => Ok(v),
        Rt::Null => Ok(Value::Null),
        _ => Err(Error::Unsupported("sort key result")),
    }
}

fn eval_usize(conn: &Connection, expr: &Expr, env: &BTreeMap<String, Rt>) -> Result<usize> {
    match eval_expr(conn, expr, env, None)? {
        Rt::Value(Value::I64(n)) if n >= 0 => Ok(n as usize),
        Rt::Value(Value::U64(n)) => Ok(n as usize),
        Rt::Value(Value::I64(_)) => Err(Error::Schema {
            message: "skip/take count must be non-negative".into(),
            span: None,
            hint: None,
        }),
        _ => Err(Error::Schema {
            message: "skip/take expects an integer".into(),
            span: None,
            hint: None,
        }),
    }
}

fn eval_binary(
    conn: &Connection,
    op: BinaryOp,
    left: &Expr,
    right: &Expr,
    env: &BTreeMap<String, Rt>,
    row_ctx: Option<&Row>,
) -> Result<Rt> {
    let l = eval_expr(conn, left, env, row_ctx)?;
    let r = eval_expr(conn, right, env, row_ctx)?;
    match op {
        BinaryOp::And => Ok(Rt::Value(Value::Bool(rt_truthy(&l)? && rt_truthy(&r)?))),
        BinaryOp::Or => Ok(Rt::Value(Value::Bool(rt_truthy(&l)? || rt_truthy(&r)?))),
        BinaryOp::Eq => Ok(Rt::Value(Value::Bool(rt_values_eq(&l, &r)?))),
        BinaryOp::Ne => Ok(Rt::Value(Value::Bool(!rt_values_eq(&l, &r)?))),
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            let lv = rt_as_value(&l)?;
            let rv = rt_as_value(&r)?;
            let ord = cmp_value(&lv, &rv);
            let ok = match op {
                BinaryOp::Lt => ord == core::cmp::Ordering::Less,
                BinaryOp::Le => ord != core::cmp::Ordering::Greater,
                BinaryOp::Gt => ord == core::cmp::Ordering::Greater,
                BinaryOp::Ge => ord != core::cmp::Ordering::Less,
                _ => unreachable!(),
            };
            Ok(Rt::Value(Value::Bool(ok)))
        }
        BinaryOp::Add => match (rt_as_value(&l)?, rt_as_value(&r)?) {
            (Value::Text(a), Value::Text(b)) => Ok(Rt::Value(Value::Text(a + &b))),
            (Value::I64(a), Value::I64(b)) => Ok(Rt::Value(Value::I64(a + b))),
            _ => Err(Error::Unsupported("add operand types")),
        },
        BinaryOp::Sub => match (rt_as_value(&l)?, rt_as_value(&r)?) {
            (Value::I64(a), Value::I64(b)) => Ok(Rt::Value(Value::I64(a - b))),
            _ => Err(Error::Unsupported("sub operand types")),
        },
        BinaryOp::Mul => match (rt_as_value(&l)?, rt_as_value(&r)?) {
            (Value::I64(a), Value::I64(b)) => Ok(Rt::Value(Value::I64(a * b))),
            _ => Err(Error::Unsupported("mul operand types")),
        },
        BinaryOp::Div => match (rt_as_value(&l)?, rt_as_value(&r)?) {
            (Value::I64(_), Value::I64(0)) => Err(Error::Schema {
                message: "division by zero".into(),
                span: None,
                hint: None,
            }),
            (Value::I64(a), Value::I64(b)) => Ok(Rt::Value(Value::I64(a / b))),
            _ => Err(Error::Unsupported("div operand types")),
        },
        _ => Err(Error::Unsupported("binary operator")),
    }
}

fn literal_to_value(literal: &Literal) -> Result<Value> {
    match literal {
        Literal::Null => Ok(Value::Null),
        Literal::Bool(v) => Ok(Value::Bool(*v)),
        Literal::Int(text) => {
            let n: i64 = text.parse().map_err(|_| Error::Schema {
                message: format!("invalid integer literal `{text}`"),
                span: None,
                hint: None,
            })?;
            Ok(Value::I64(n))
        }
        Literal::Float(text) => {
            let n: f64 = text.parse().map_err(|_| Error::Schema {
                message: format!("invalid float literal `{text}`"),
                span: None,
                hint: None,
            })?;
            Ok(Value::F64(n))
        }
        Literal::String(s) => Ok(Value::Text(s.clone())),
        Literal::Ident(name) => Ok(Value::Text(name.clone())),
        _ => Err(Error::Unsupported("literal kind")),
    }
}

fn rt_truthy(rt: &Rt) -> Result<bool> {
    match rt {
        Rt::Value(Value::Bool(b)) => Ok(*b),
        Rt::Value(Value::Null) | Rt::Null => Ok(false),
        Rt::Value(_) => Ok(true),
        Rt::Row(_) | Rt::Draft { .. } | Rt::Drafts { .. } | Rt::Rows(_) | Rt::Plan(_) => Ok(true),
    }
}

fn rt_as_value(rt: &Rt) -> Result<Value> {
    match rt {
        Rt::Value(v) => Ok(v.clone()),
        Rt::Null => Ok(Value::Null),
        _ => Err(Error::Schema {
            message: "expected a scalar value".into(),
            span: None,
            hint: None,
        }),
    }
}

fn rt_values_eq(left: &Rt, right: &Rt) -> Result<bool> {
    Ok(values_eq(&rt_as_value(left)?, &rt_as_value(right)?))
}

fn values_eq(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::I64(a), Value::I64(b)) => a == b,
        (Value::U64(a), Value::U64(b)) => a == b,
        (Value::I64(a), Value::U64(b)) => *a >= 0 && (*a as u64) == *b,
        (Value::U64(a), Value::I64(b)) => *b >= 0 && *a == (*b as u64),
        (Value::F64(a), Value::F64(b)) => a == b,
        (Value::Text(a), Value::Text(b)) => a == b,
        (Value::Uuid(a), Value::Uuid(b)) => a == b,
        (Value::Uuid(a), Value::Text(b)) | (Value::Text(b), Value::Uuid(a)) => {
            uuid::Uuid::parse_str(b).ok() == Some(*a)
        }
        _ => false,
    }
}

fn cmp_value(left: &Value, right: &Value) -> core::cmp::Ordering {
    match (left, right) {
        (Value::I64(a), Value::I64(b)) => a.cmp(b),
        (Value::U64(a), Value::U64(b)) => a.cmp(b),
        (Value::Text(a), Value::Text(b)) => a.cmp(b),
        (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
        (Value::Uuid(a), Value::Uuid(b)) => a.as_bytes().cmp(b.as_bytes()),
        (Value::Null, Value::Null) => core::cmp::Ordering::Equal,
        (Value::Null, _) => core::cmp::Ordering::Less,
        (_, Value::Null) => core::cmp::Ordering::Greater,
        _ => core::cmp::Ordering::Equal,
    }
}

fn cmp_opt_value(left: Option<&Value>, right: Option<&Value>) -> core::cmp::Ordering {
    match (left, right) {
        (None, None) => core::cmp::Ordering::Equal,
        (None, Some(_)) => core::cmp::Ordering::Less,
        (Some(_), None) => core::cmp::Ordering::Greater,
        (Some(a), Some(b)) => cmp_value(a, b),
    }
}
