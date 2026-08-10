# YYDB single-file format (P0)

**Product rule:** one authoritative database file `app.yydb`. Optional sidecars
only for WAL:

```text
app.yydb                 # primary database file (truth after checkpoint)
app.yydb-wal             # WAL frames when journal_mode=wal
app.yydb-shm             # WAL coordination / frame index
app.yydb.objects/…       # recommended CAS for large bytes (not a second DB)
```

Do **not** introduce standing sidecars such as `.yypg` or `.yydb-vec`.

VOS is the only schema/query language. This document is the **storage** layout,
not a query dialect. See
[`vos-language/documentation/no-sql-invariant.md`](../../vos-language/documentation/no-sql-invariant.md).

## Two on-disk eras (migration policy)

| Era | Magic | Role |
| --- | --- | --- |
| **Snapshot (compat)** | `YYDB\x01` | Whole-file encoded `State` (schema + KV). WAL stores full-state snapshot frames. Create via `OpenFlags::create_snapshot`. |
| **Paged (default create)** | `YYDB\x02` | Fixed 4 KiB pages. Catalog / rows / indexes. Same product as snapshot; layout version only. |

`Connection::open` / `OpenFlags::new()` create `YYDB\x02`. Existing `YYDB\x01`
files still open by magic. Migration policy:

1. New files default to `YYDB\x02` only.
2. Opening `YYDB\x01` remains supported until an explicit one-shot migrator
   rewrites to paged form (not silent dual-write).
3. File **format version** (meta `format_version`) is independent of the VOS
   **schema version** stored in the catalog.

## Paged file: page size

- `PAGE_SIZE = 4096`
- Last 4 bytes of every page: CRC-32 (ISO-HDLC) over bytes `[0, PAGE_USABLE)`
- `PAGE_USABLE = 4092` — B+Tree cell packing must not use the checksum trailer

## Meta page (page 0)

| Offset | Size | Field |
| --- | ---: | --- |
| 0 | 5 | Magic `YYDB\x02` |
| 5 | 3 | Reserved (zero) |
| 8 | 4 | `page_count` (u32 LE), includes meta |
| 12 | 4 | `btree_root` (u32 LE), `0` = unset |
| 16 | 4 | `freelist_head` (u32 LE), `0` = empty |
| 20 | 4 | `format_version` (u32 LE), starts at `1` |
| 24 | 4 | `catalog_root` page id (u32 LE), `0` = empty |
| 4092 | 4 | Page CRC-32 |

## Catalog pages (type `0x0c`)

Schema + field-identity catalog blob live on one or more catalog pages
pointed to by `catalog_root`. Chained pages use `has_next` (byte 1) and
`next` (bytes 4..8). Payload header on each page:

| Offset | Size | Field |
| --- | ---: | --- |
| 0 | 1 | Page type `0x0c` |
| 1 | 1 | `has_next` (0/1) |
| 4 | 4 | `next` page id when chained |
| 8 | 4 | Total payload length (all pages) |
| 12 | 4 | This page's chunk length |
| 24 | … | Chunk bytes |

Decoded payload: `[schema marker][version][document][catalog marker][catalog blob]`.

Primary KV records (UTF-8 keys → byte values) live in the B+Tree rooted at
`btree_root`. `Connection` on paged (`YYDB\x02`) files loads and commits catalog +
records via catalog pages and this btree.

### Key namespaces (same on snapshot and paged)

| Prefix | Role |
| --- | --- |
| `row/<table>/<pk-token>` | Typed table row payload |
| `idx/<table>/<field>/<value-token>` | Unique secondary index → primary-key token bytes |
| (other UTF-8 keys) | Opaque KV (`Connection::put` / `get`) |

Unique indexes share the primary btree (no separate index root). Dedicated
index pages remain a later optimization.

New files: `Connection::open` / `OpenFlags::new()` / `OpenFlags::wal()` create
`YYDB\x02`. Use `OpenFlags::create_snapshot` (or `create_snapshot_wal`) for
`YYDB\x01`. Existing files always open by magic.

## Free list

Freed pages are pushed onto a singly linked list headed by `freelist_head`.
A free page stores `next_free` (u32 LE) at offset 0 of the usable region.
`allocate_page` pops the freelist before extending the file.

## B+Tree pages

Page type byte at offset 0 (`0x0d` leaf, `0x05` internal). Algorithms live in
`backends/yydb/src/btree.rs`. Row and unique-index entries use the namespaces
above inside the single `btree_root` tree.

## Checksums

- On `write_page`, recompute and store the trailer CRC.
- On `read_page`, verify; mismatch → `Error::Corrupt`.
- Truncated files (`len < page_count * PAGE_SIZE`) → `Error::Corrupt`.

## WAL recovery (P1 slice)

WAL header magic: `YYWL\x01`. Frames are `[type][u32 LE len][payload]`.

### Approach (current): full-state snapshot frames (type `1`)

Both snapshot (`YYDB\x01`) and paged (`YYDB\x02`) connections use **type-1
full-state frames**. Payload bytes are the same encoded `State` blob used by
the snapshot main file (`YYDB\x01` + schema/records). For paged files this is
deliberately **not** a page-image log yet:

- Commits in `journal_mode=wal` append one complete type-1 frame and leave the
  main `.yydb` unchanged until `checkpoint`.
- On reopen, the main file is loaded (snapshot bytes or paged catalog+btree),
  then WAL frames are applied in order (**last complete frame wins**).
- A **truncated / junk tail** is discarded — including a complete `type`+`len`
  with a torn mid-body payload, a tear inside the length field, or non-type-1
  garbage after the last complete frame. The WAL is truncated to the last
  complete byte offset and SHM is refreshed (soft-repair). Reopen never yields
  a half-applied schema/data mix.
- SHM (`n_frames` / `wal_bytes`) is **always resynced** from the scanned
  durable prefix on replay, so metadata cannot stay ahead of frames after a
  crash between append and SHM update.
- `checkpoint` folds the recovered state into the main file (snapshot rewrite
  or paged `write_paged_state`) **first**, then truncates `-wal` / resets
  `-shm`. The intentional crash window is therefore **main already folded +
  WAL still present**: reopen applies last-complete-frame-wins and remains
  consistent with the folded state (same type-1 payloads). Truncating WAL
  before writing main is **not** a supported recovery path for type-1 frames.

**Why not page-frame WAL yet:** type-1 frames reuse the existing encode/replay
path and keep crash reopen consistent with one `.yydb` + `-wal`/`-shm`. A
future page-frame WAL (dirty page images + meta) is preferred long-term for
I/O, but is out of scope for this slice.

### Crash-point coverage (tests)

| Crash point | Semantics | Status |
| --- | --- | --- |
| Reopen after commit, no checkpoint | Truth from WAL frames | Covered |
| Truncated length field (torn header) | Soft-repair to last complete frame | Covered |
| Torn mid-body (complete type+len, short payload) | Soft-repair to last complete frame | Covered |
| Junk / unsupported type after last complete frame | Soft-repair; truncate dirty tail | Covered |
| Empty / header-only WAL | Keep main file state | Covered |
| Checkpoint then delete sidecars | Main holds folded state | Covered |
| Mid-checkpoint: main folded, WAL retained | Last frame wins; consistent with main | Covered |
| SHM stale-high vs durable WAL | SHM resynced on replay | Covered |
| Torn / corrupt main mid-write (paged) while WAL present | Needs durable page images or atomic main rewrite | **Gap** (page-frame WAL) |
| CAS `objects/` vs main/WAL commit skew | Objects store is beside-db; not fenced with WAL | **Gap** |
| Page-frame WAL | Preferred long-term journal | **Gap** |

Covered crash points (tests): reopen after commit without checkpoint; truncated
tail soft-repair (length tear + mid-body tear + junk after last good); empty
(header-only) WAL; checkpoint then remove sidecars; mid-checkpoint
main-folded/WAL-retained; stale-high SHM resync. Remaining P1 gaps: paged
torn-main mid-write, CAS commit skew, page-frame WAL.

## Version counters

| Counter | Meaning |
| --- | --- |
| `format_version` | On-disk page layout revision |
| VOS schema `version` / `DdlRevision` | Logical schema identity inside the catalog |
| WAL SHM `n_frames` | Incomplete coordination metadata only |
