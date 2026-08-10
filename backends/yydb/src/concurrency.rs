//! Exclusive write gate + shared readers (concurrency slices).
//!
//! ## Model
//!
//! - One **exclusive writer** per canonical `.yydb` path.
//! - **Process-local** owner slot: `begin` / DDL take a long lease; autocommit
//!   writes take a short lease. Contended writers fail fast with
//!   [`Error::Busy`] — no blocking wait, no multi-writer.
//! - **Shared readers** (process-local): any number of connections may read
//!   while no writer holds the lease. A peer read while another connection
//!   holds the write lease returns [`Error::Busy`]. Acquiring the write lease
//!   fails with [`Error::Busy`] while peer readers are active (prevents
//!   mid-write torn reads in-process). The writer may read its own connection
//!   without bumping the reader count.
//! - **Cross-process**: the write lease also holds an OS exclusive lock on
//!   `{db}-write.lock` (non-blocking `try_lock`). Another process that tries
//!   to write while the lock is held gets [`Error::Busy`].
//!
//! ## Still unsafe (documented, not fixed here)
//!
//! - No MVCC / snapshot isolation (readers see last committed file image only
//!   when no writer is active).
//! - Cross-process readers are **not** fenced by the OS lock (only writers).
//! - CAS `objects/` is not fenced by this gate.
//! - No blocking wait / lock timeouts.
//!
//! See `documentation/concurrency.md`.

use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, OnceLock,
    },
};

use fs4::fs_std::FileExt;
use yydb_types::{Error, Result};

static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(1);
static GATES: OnceLock<Mutex<HashMap<PathBuf, Arc<WriteGate>>>> = OnceLock::new();

#[derive(Debug, Default)]
struct GateInner {
    writer: Option<u64>,
    /// Active process-local readers (connections other than the writer).
    readers: u32,
}

/// Exclusive write owner + shared readers + OS lock for one database path.
#[derive(Debug)]
pub(crate) struct WriteGate {
    inner: Mutex<GateInner>,
    lock_path: PathBuf,
    os_lock: Mutex<Option<File>>,
}

/// RAII ticket from [`WriteGate::try_enter_read`]; releases the reader slot on drop.
#[derive(Debug)]
pub(crate) struct ReadTicket<'a> {
    gate: &'a WriteGate,
    counted: bool,
}

impl Drop for ReadTicket<'_> {
    fn drop(&mut self) {
        if !self.counted {
            return;
        }
        let mut inner = self
            .gate
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        inner.readers = inner.readers.saturating_sub(1);
    }
}

impl WriteGate {
    fn new(lock_path: PathBuf) -> Self {
        Self {
            inner: Mutex::new(GateInner::default()),
            lock_path,
            os_lock: Mutex::new(None),
        }
    }

    /// Acquire exclusive write for `conn_id`. Re-entrant for the same connection.
    ///
    /// Fails if another writer holds the lease, or if peer readers are active.
    pub(crate) fn try_acquire(&self, conn_id: u64) -> Result<()> {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match inner.writer {
            Some(id) if id == conn_id => Ok(()),
            Some(_) => Err(Error::Busy {
                message: "database write lease held by another connection".into(),
            }),
            None if inner.readers > 0 => Err(Error::Busy {
                message: "database readers active; writer cannot start".into(),
            }),
            None => {
                self.try_lock_os()?;
                inner.writer = Some(conn_id);
                Ok(())
            }
        }
    }

    pub(crate) fn release(&self, conn_id: u64) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if inner.writer == Some(conn_id) {
            inner.writer = None;
            drop(inner);
            self.unlock_os();
        }
    }

    /// Enter a shared read. Peer writers block this; the active writer may read.
    pub(crate) fn try_enter_read(&self, conn_id: u64) -> Result<ReadTicket<'_>> {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match inner.writer {
            Some(id) if id != conn_id => Err(Error::Busy {
                message: "database write lease held; readers blocked until release".into(),
            }),
            Some(_) => Ok(ReadTicket {
                gate: self,
                counted: false,
            }),
            None => {
                inner.readers = inner.readers.saturating_add(1);
                Ok(ReadTicket {
                    gate: self,
                    counted: true,
                })
            }
        }
    }

    fn try_lock_os(&self) -> Result<()> {
        let mut held = self
            .os_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if held.is_some() {
            return Ok(());
        }
        if let Some(parent) = self.lock_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&self.lock_path)
            .map_err(Error::Io)?;
        match file.try_lock_exclusive() {
            Ok(true) => {
                *held = Some(file);
                Ok(())
            }
            Ok(false) => Err(Error::Busy {
                message: "database write lock held by another process".into(),
            }),
            Err(err) => Err(Error::Io(err)),
        }
    }

    fn unlock_os(&self) {
        let mut held = self
            .os_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(file) = held.take() {
            let _ = file.unlock();
            drop(file);
        }
    }
}

pub(crate) fn next_conn_id() -> u64 {
    NEXT_CONN_ID.fetch_add(1, Ordering::Relaxed)
}

/// `{db}-write.lock` sidecar used for cross-process exclusive write locking.
pub fn write_lock_path(db: &Path) -> PathBuf {
    let mut os = db.as_os_str().to_owned();
    os.push("-write.lock");
    PathBuf::from(os)
}

/// Gate for an existing file path (canonicalized when possible).
pub(crate) fn gate_for_path(path: &Path) -> Arc<WriteGate> {
    let key = lock_key(path);
    let lock_path = write_lock_path(&key);
    let map = GATES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    guard
        .entry(key)
        .or_insert_with(|| Arc::new(WriteGate::new(lock_path)))
        .clone()
}

fn lock_key(path: &Path) -> PathBuf {
    if let Ok(canon) = fs::canonicalize(path) {
        return canon;
    }
    if path.is_absolute() {
        path.to_path_buf()
    } else if let Ok(cwd) = std::env::current_dir() {
        cwd.join(path)
    } else {
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(label: &str) -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "yydb-{label}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("db.yydb");
        fs::write(&path, b"x").unwrap();
        (dir, path)
    }

    #[test]
    fn same_conn_reentrant_second_conn_busy_then_release() {
        let (dir, path) = temp_path("gate-unit");
        let gate = gate_for_path(&path);
        gate.try_acquire(1).unwrap();
        gate.try_acquire(1).unwrap();
        assert!(gate.try_acquire(2).unwrap_err().is_busy());
        gate.release(1);
        gate.try_acquire(2).unwrap();
        gate.release(2);
        let _ = fs::remove_file(write_lock_path(&path));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn peer_read_busy_while_writer_holds_lease() {
        let (dir, path) = temp_path("rw-read");
        let gate = gate_for_path(&path);
        gate.try_acquire(1).unwrap();
        assert!(gate.try_enter_read(2).unwrap_err().is_busy());
        gate.try_enter_read(1).unwrap(); // writer may read
        gate.release(1);
        let _ticket = gate.try_enter_read(2).unwrap();
        let _ = fs::remove_file(write_lock_path(&path));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn writer_busy_while_peer_reader_active() {
        let (dir, path) = temp_path("rw-write");
        let gate = gate_for_path(&path);
        let ticket = gate.try_enter_read(2).unwrap();
        let err = gate.try_acquire(1).unwrap_err();
        assert!(err.is_busy());
        assert!(err.to_string().contains("readers active"));
        drop(ticket);
        gate.try_acquire(1).unwrap();
        gate.release(1);
        let _ = fs::remove_file(write_lock_path(&path));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn gate_for_path_shares_canonical_key() {
        let (dir, path) = temp_path("gate-share");
        let a = gate_for_path(&path);
        let b = gate_for_path(&path);
        assert!(Arc::ptr_eq(&a, &b));
        a.try_acquire(7).unwrap();
        assert!(b.try_acquire(8).unwrap_err().is_busy());
        a.release(7);
        let _ = fs::remove_file(write_lock_path(&path));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn os_lock_busy_when_foreign_process_holds_file() {
        let (dir, path) = temp_path("oslock");
        let lock = write_lock_path(&path);
        let foreign = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&lock)
            .unwrap();
        assert!(foreign.try_lock_exclusive().unwrap());

        let gate = gate_for_path(&path);
        let err = gate.try_acquire(1).unwrap_err();
        assert!(err.is_busy());
        assert!(err.to_string().contains("another process"));

        drop(foreign);
        gate.try_acquire(1).unwrap();
        gate.release(1);
        let _ = fs::remove_file(&lock);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }
}
