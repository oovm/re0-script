//! Fixed-size page I/O for the paged `.yydb` layout.
//!
//! See [`documentation/file-format.md`](../../../documentation/file-format.md).
//! New files default to paged (`YYDB\x02`); snapshot (`YYDB\x01`) remains
//! openable for compat via [`crate::OpenFlags::create_snapshot`].

use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use yydb_types::{Error, Result};

/// Default page size (4 KiB).
pub const PAGE_SIZE: usize = 4096;

/// Bytes reserved at the end of every page for CRC-32.
pub const PAGE_CHECKSUM_LEN: usize = 4;

/// Usable payload bytes per page (`PAGE_SIZE - PAGE_CHECKSUM_LEN`).
pub const PAGE_USABLE: usize = PAGE_SIZE - PAGE_CHECKSUM_LEN;

/// Paged `.yydb` magic (`YYDB` + layout version byte `\x02`).
pub const PAGER_MAGIC: &[u8; 5] = b"YYDB\x02";

/// Current on-disk format version stored in the meta page.
pub const FORMAT_VERSION: u32 = 1;

/// Meta: `page_count` (u32 LE).
pub const META_PAGE_COUNT_OFFSET: usize = 8;
/// Meta: B+Tree root page id (u32 LE), `0` = unset.
pub const META_BTREE_ROOT_OFFSET: usize = 12;
/// Meta: freelist head page id (u32 LE), `0` = empty.
pub const META_FREELIST_HEAD_OFFSET: usize = 16;
/// Meta: file format version (u32 LE).
pub const META_FORMAT_VERSION_OFFSET: usize = 20;
/// Meta: catalog root page id (u32 LE), `0` = empty catalog.
pub const META_CATALOG_ROOT_OFFSET: usize = 24;

/// Read catalog root from the meta page (`0` = empty).
pub fn read_catalog_root(pager: &mut Pager) -> Result<PageId> {
    let meta = pager.read_page(0)?;
    Ok(u32::from_le_bytes(
        meta.data[META_CATALOG_ROOT_OFFSET..META_CATALOG_ROOT_OFFSET + 4]
            .try_into()
            .unwrap(),
    ))
}

/// Persist catalog root on the meta page.
pub fn write_catalog_root(pager: &mut Pager, root: PageId) -> Result<()> {
    let mut meta = pager.read_page(0)?;
    meta.data[META_CATALOG_ROOT_OFFSET..META_CATALOG_ROOT_OFFSET + 4]
        .copy_from_slice(&root.to_le_bytes());
    pager.write_page(&meta)
}

/// Zero-based page number. Page 0 is the meta page.
pub type PageId = u32;

/// One in-memory page image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    /// Page number.
    pub id: PageId,
    /// Raw page bytes (`PAGE_SIZE`), including checksum trailer.
    pub data: Box<[u8; PAGE_SIZE]>,
}

impl Page {
    /// Allocate a zero-filled page.
    pub fn zeroed(id: PageId) -> Self {
        Self {
            id,
            data: Box::new([0; PAGE_SIZE]),
        }
    }
}

/// File-backed pager for the paged `.yydb` era.
#[derive(Debug)]
pub struct Pager {
    path: PathBuf,
    file: File,
    page_count: u32,
}

impl Pager {
    /// Create a new pager file with a meta page.
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)?;
        let mut meta = Page::zeroed(0);
        meta.data[..5].copy_from_slice(PAGER_MAGIC);
        write_u32(meta.data.as_mut_slice(), META_PAGE_COUNT_OFFSET, 1);
        write_u32(
            meta.data.as_mut_slice(),
            META_FORMAT_VERSION_OFFSET,
            FORMAT_VERSION,
        );
        seal_checksum(&mut meta.data);
        file.write_all(&*meta.data)?;
        file.flush()?;
        Ok(Self {
            path,
            file,
            page_count: 1,
        })
    }

    /// Open an existing pager file.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = OpenOptions::new().read(true).write(true).open(&path)?;
        let len = file.metadata()?.len();
        // Short / mid-write torn mains must surface as Corrupt so openers can
        // rebuild from type-1 WAL (UnexpectedEof would skip that path).
        if len < PAGE_SIZE as u64 {
            return Err(Error::Corrupt("pager file truncated"));
        }
        let mut meta = [0u8; PAGE_SIZE];
        file.read_exact(&mut meta)?;
        verify_checksum(&meta)?;
        if meta[..5] != *PAGER_MAGIC {
            return Err(Error::Corrupt("unknown pager magic"));
        }
        let page_count = read_u32(&meta, META_PAGE_COUNT_OFFSET);
        if page_count == 0 {
            return Err(Error::Corrupt("pager page_count is zero"));
        }
        let format_version = read_u32(&meta, META_FORMAT_VERSION_OFFSET);
        if format_version == 0 {
            // Pre-checksum prototypes left this field zero; treat as v1.
        } else if format_version > FORMAT_VERSION {
            return Err(Error::Corrupt("pager format_version too new"));
        }
        let expected = u64::from(page_count) * PAGE_SIZE as u64;
        if len < expected {
            return Err(Error::Corrupt("pager file truncated"));
        }
        Ok(Self {
            path,
            file,
            page_count,
        })
    }

    /// Filesystem path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Number of pages including the meta page.
    pub fn page_count(&self) -> u32 {
        self.page_count
    }

    /// File format version from the meta page.
    pub fn format_version(&mut self) -> Result<u32> {
        let meta = self.read_page(0)?;
        let v = read_u32(meta.data.as_slice(), META_FORMAT_VERSION_OFFSET);
        Ok(if v == 0 { FORMAT_VERSION } else { v })
    }

    /// Freelist head (`0` = empty).
    pub fn freelist_head(&mut self) -> Result<PageId> {
        let meta = self.read_page(0)?;
        Ok(read_u32(meta.data.as_slice(), META_FREELIST_HEAD_OFFSET))
    }

    /// Read page `id` (verifies CRC).
    ///
    /// If `id` is beyond this handle's remembered `page_count` (another
    /// connection may have extended the file), refresh from the meta page on
    /// disk first. This is **not** MVCC — only makes committed peer pages
    /// reachable under the process-local write lease.
    pub fn read_page(&mut self, id: PageId) -> Result<Page> {
        if id >= self.page_count && id != 0 {
            self.read_page(0)?;
        }
        if id >= self.page_count {
            return Err(Error::Corrupt("page id out of range"));
        }
        let mut page = Page::zeroed(id);
        let offset = u64::from(id) * PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(&mut *page.data)?;
        verify_checksum(&page.data)?;
        if id == 0 {
            let count = read_u32(page.data.as_slice(), META_PAGE_COUNT_OFFSET);
            if count == 0 {
                return Err(Error::Corrupt("pager page_count is zero"));
            }
            // Take the max so (1) peer extensions become visible and (2) a
            // local allocate that has bumped `page_count` but not yet flushed
            // meta is not clobbered by a mid-allocate meta read.
            self.page_count = self.page_count.max(count);
        }
        Ok(page)
    }

    /// Overwrite page `id` (must already exist). Seals CRC before write.
    pub fn write_page(&mut self, page: &Page) -> Result<()> {
        if page.id >= self.page_count {
            return Err(Error::Corrupt("page id out of range"));
        }
        let mut data = *page.data;
        seal_checksum(&mut data);
        let offset = u64::from(page.id) * PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&data)?;
        self.file.flush()?;
        if page.id == 0 {
            self.page_count = read_u32(&data, META_PAGE_COUNT_OFFSET);
        }
        Ok(())
    }

    /// Allocate a page: prefer freelist, otherwise append.
    pub fn allocate_page(&mut self) -> Result<PageId> {
        // Pull peer-committed page_count / freelist before allocating.
        self.read_page(0)?;
        let head = self.freelist_head()?;
        if head != 0 {
            if head >= self.page_count || head == 0 {
                return Err(Error::Corrupt("invalid freelist head"));
            }
            let recycled = self.read_page(head)?;
            let next = read_u32(recycled.data.as_slice(), 0);
            let mut meta = self.read_page(0)?;
            write_u32(meta.data.as_mut_slice(), META_FREELIST_HEAD_OFFSET, next);
            self.write_page(&meta)?;
            let blank = Page::zeroed(head);
            self.write_page(&blank)?;
            return Ok(head);
        }

        let id = self.page_count;
        let page = Page::zeroed(id);
        let offset = u64::from(id) * PAGE_SIZE as u64;
        let mut data = *page.data;
        seal_checksum(&mut data);
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&data)?;
        self.page_count += 1;
        let mut meta = self.read_page(0)?;
        write_u32(
            meta.data.as_mut_slice(),
            META_PAGE_COUNT_OFFSET,
            self.page_count,
        );
        self.write_page(&meta)?;
        Ok(id)
    }

    /// Truncate the file and rewrite a fresh meta page (WAL torn-main repair).
    ///
    /// Callers must re-init catalog / btree afterward (`catalog_page::init_empty`
    /// then rewrite committed state). Used when the paged main image is
    /// truncated or checksum-corrupt but type-1 WAL frames are complete.
    pub fn reformat(&mut self) -> Result<()> {
        self.file.set_len(0)?;
        self.file.seek(SeekFrom::Start(0))?;
        let mut meta = Page::zeroed(0);
        meta.data[..5].copy_from_slice(PAGER_MAGIC);
        write_u32(meta.data.as_mut_slice(), META_PAGE_COUNT_OFFSET, 1);
        write_u32(
            meta.data.as_mut_slice(),
            META_FORMAT_VERSION_OFFSET,
            FORMAT_VERSION,
        );
        seal_checksum(&mut meta.data);
        self.file.write_all(&*meta.data)?;
        self.file.flush()?;
        self.page_count = 1;
        Ok(())
    }

    /// Return `id` to the freelist (`id` must not be the meta page).
    pub fn free_page(&mut self, id: PageId) -> Result<()> {
        if id == 0 {
            return Err(Error::Corrupt("cannot free meta page"));
        }
        if id >= self.page_count {
            return Err(Error::Corrupt("page id out of range"));
        }
        let head = self.freelist_head()?;
        let mut page = Page::zeroed(id);
        write_u32(page.data.as_mut_slice(), 0, head);
        self.write_page(&page)?;
        let mut meta = self.read_page(0)?;
        write_u32(meta.data.as_mut_slice(), META_FREELIST_HEAD_OFFSET, id);
        self.write_page(&meta)?;
        Ok(())
    }
}

fn write_u32(buf: &mut [u8], at: usize, value: u32) {
    buf[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

fn read_u32(buf: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(buf[at..at + 4].try_into().unwrap())
}

fn seal_checksum(page: &mut [u8; PAGE_SIZE]) {
    let sum = crc32(&page[..PAGE_USABLE]);
    page[PAGE_USABLE..PAGE_SIZE].copy_from_slice(&sum.to_le_bytes());
}

fn verify_checksum(page: &[u8; PAGE_SIZE]) -> Result<()> {
    let expected = u32::from_le_bytes(page[PAGE_USABLE..PAGE_SIZE].try_into().unwrap());
    let actual = crc32(&page[..PAGE_USABLE]);
    if expected != actual {
        return Err(Error::Corrupt("page checksum mismatch"));
    }
    Ok(())
}

/// CRC-32/ISO-HDLC (poly `0xEDB88320`).
fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_pager(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("yydb-pager-{label}-{nonce}.yydb"))
    }

    #[test]
    fn create_allocate_roundtrip() {
        let path = temp_pager("basic");
        let mut pager = Pager::create(&path).unwrap();
        assert_eq!(pager.page_count(), 1);
        assert_eq!(pager.format_version().unwrap(), FORMAT_VERSION);
        let id = pager.allocate_page().unwrap();
        assert_eq!(id, 1);
        let mut page = pager.read_page(id).unwrap();
        page.data[0] = 0xAB;
        pager.write_page(&page).unwrap();
        drop(pager);

        let mut reopened = Pager::open(&path).unwrap();
        assert_eq!(reopened.page_count(), 2);
        let page = reopened.read_page(1).unwrap();
        assert_eq!(page.data[0], 0xAB);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn freelist_reuses_freed_pages() {
        let path = temp_pager("free");
        let mut pager = Pager::create(&path).unwrap();
        let a = pager.allocate_page().unwrap();
        let b = pager.allocate_page().unwrap();
        assert_eq!((a, b), (1, 2));
        pager.free_page(a).unwrap();
        assert_eq!(pager.freelist_head().unwrap(), a);
        let reused = pager.allocate_page().unwrap();
        assert_eq!(reused, a);
        assert_eq!(pager.page_count(), 3);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn second_pager_sees_peer_allocated_pages() {
        let path = temp_pager("peer");
        let mut a = Pager::create(&path).unwrap();
        let mut b = Pager::open(&path).unwrap();
        assert_eq!(a.page_count(), 1);
        assert_eq!(b.page_count(), 1);
        let id = a.allocate_page().unwrap();
        assert_eq!(id, 1);
        let mut page = a.read_page(id).unwrap();
        page.data[0] = 0x7E;
        a.write_page(&page).unwrap();
        let seen = b.read_page(id).unwrap();
        assert_eq!(seen.data[0], 0x7E);
        assert_eq!(b.page_count(), 2);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn checksum_detects_corruption() {
        let path = temp_pager("crc");
        let mut pager = Pager::create(&path).unwrap();
        let id = pager.allocate_page().unwrap();
        drop(pager);

        let mut bytes = std::fs::read(&path).unwrap();
        let offset = usize::try_from(id).unwrap() * PAGE_SIZE;
        bytes[offset] ^= 0xff;
        std::fs::write(&path, &bytes).unwrap();

        let mut pager = Pager::open(&path).unwrap();
        assert!(matches!(
            pager.read_page(id),
            Err(Error::Corrupt("page checksum mismatch"))
        ));
        let _ = std::fs::remove_file(&path);
    }
}
