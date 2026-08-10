# AGENTS.md — YYDB

## Authority

- Language contract: [`vos-language`](https://github.com/voml/vos-language) (`dev`), especially [`documentation/no-sql-invariant.md`](https://github.com/voml/vos-language/blob/dev/documentation/no-sql-invariant.md).
- This repo implements the **single-file** `.yydb` product. Do not merge YYDS distributed semantics here.

## Hard invariant (VOS / YYDB / YYDS)

**VOS replaces SQL.** It is not a SQL frontend, transpiler, or dialect. Formal
YYDB surfaces must not introduce SQL.

- DDL, queries, writes, and index operations use **VOS semantics only**.
- Execution: `.vos` → parser → AST → semantic analysis → operation/query IR → YYDB executor.
- Do **not** add SQL parser/AST, SQL-to-VOS conversion, SQL endpoints, or
  “compile VOS to SQL then run.”
- Do **not** grow features by designing SQL first.

## Product facts

- One portable `.yydb` (+ optional WAL/SHM); recommended large bytes via CAS `objects/`.
- Database truth / pull mode; not ArtGPT local-truth push-to-MySQL.
- Embed preferentially via Rust; TS product surface is `@yydb/yydb`.
- Depend on `vos` git `@ dev`; no parallel schema/query dialect.
- On-disk layout authority: [`documentation/file-format.md`](./documentation/file-format.md).
  New files default to paged era (`YYDB\x02` via `pager`/`btree`). Snapshot era
  (`YYDB\x01`) remains openable; create via `OpenFlags::create_snapshot`.

## Near-term priorities

1. **P0** — own `.yydb` via pager (catalog/row/index pages), no new sidecars. *(default create = paged)*
2. **P1** — crash-point recovery matrix (torn/junk/mid-ckpt soft-repair in; paged torn-main + page-frame WAL next).
3. **P2+** — concurrency proofs, formal migration, VOS query executor, Spark ProjectStore.

