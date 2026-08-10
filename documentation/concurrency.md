# YYDB concurrency

## What is guaranteed

### Process-local write lease

Within **one process**, file-backed connections on the same canonical `.yydb`
path share an exclusive **write lease**:

| Operation | Lease |
| --- | --- |
| `begin` … `commit` / `rollback` | Long lease for the whole data transaction |
| `begin_ddl_session` … commit / drop | Long lease for the DDL session |
| Autocommit writes (`put`, `ensure_schema`, `checkpoint`, …) | Short lease for the durable write only |

A second connection that tries to write while the lease is held receives
`Error::Busy` immediately (**non-blocking**). Same-connection re-entry is
allowed. In-memory databases (`open_in_memory`) do not use a path gate.

### Process-local readers vs writers

| Situation | Result |
| --- | --- |
| No writer; any number of readers | Allowed |
| Writer holds lease; peer `get` / committed read | `Error::Busy` |
| Peer readers active; another connection starts write | `Error::Busy` |
| Writer reads on its own connection | Allowed (no reader-count bump) |

This is **not** MVCC: peers do not observe a writer's uncommitted draft, and they
do not read the file while a write lease is held (avoids in-process torn page
reads). After the lease is released, a peer read reloads from disk (paged meta
`page_count` is refreshed at the start of each committed load).

### Cross-process OS write lock

The write lease also holds an OS exclusive lock on the sidecar
`{db}-write.lock` (non-blocking `try_lock_exclusive` via `fs4`).

- Another **process** that attempts a write while the lock is held gets
  `Error::Busy` (`… held by another process`).
- Multiple processes may still **open** / **read** the same file without an OS
  read lock; only writes are serialized cross-process.
- The lock file is an optional coordination sidecar (like `-wal` / `-shm`), not
  a second database.

## What is **not** guaranteed yet

- Cross-process reader fencing (peer process may still read mid-write).
- MVCC / snapshot isolation across readers.
- Shared pager page-cache (each `Connection` has its own `Pager` handle; meta
  refresh makes committed extensions visible).
- CAS `objects/` writes are **not** fenced by the write lease.
- Blocking wait / lock timeouts (fail-fast Busy only).

## Product rule

Do not market YYDB as a multi-writer or concurrent-read-during-write database
until cross-process reader locks (and/or WAL+MVCC) land. Current slices prove
**single-writer discipline** plus **in-process reader/writer exclusion**.
