//! B+Tree pages on top of [`crate::pager`].
//!
//! This module provides the page algorithms that will back the integrated
//! `.yydb` file layout. It does not define a separate product-sidecar format.

use crate::pager::{Page, PageId, Pager, META_BTREE_ROOT_OFFSET, PAGER_MAGIC, PAGE_USABLE};
use yydb_types::{Error, Result};

/// Leaf page type byte stored at offset 0.
pub const PAGE_TYPE_LEAF: u8 = 0x0d;
/// Internal (branch) page type byte.
pub const PAGE_TYPE_INTERNAL: u8 = 0x05;

const HEADER_LEN: usize = 16;

/// In-memory view of a leaf page: sorted `(key, value)` cells.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LeafPage {
    /// Right-sibling leaf (`0` = none).
    pub next: PageId,
    /// Cells sorted ascending by key.
    pub cells: Vec<(Vec<u8>, Vec<u8>)>,
}

impl LeafPage {
    /// Empty leaf with no sibling.
    pub fn new() -> Self {
        Self::default()
    }

    /// Binary-search for `key`.
    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.cells
            .binary_search_by(|(k, _)| k.as_slice().cmp(key))
            .ok()
            .map(|i| self.cells[i].1.as_slice())
    }

    /// Remove `key` if present. Returns whether it existed.
    pub fn remove(&mut self, key: &[u8]) -> bool {
        match self.cells.binary_search_by(|(k, _)| k.as_slice().cmp(key)) {
            Ok(i) => {
                self.cells.remove(i);
                true
            }
            Err(_) => false,
        }
    }

    /// Insert or replace `key` in memory (does not check page capacity).
    pub fn upsert_unchecked(&mut self, key: Vec<u8>, value: Vec<u8>) {
        match self
            .cells
            .binary_search_by(|(k, _)| k.as_slice().cmp(key.as_slice()))
        {
            Ok(i) => self.cells[i] = (key, value),
            Err(i) => self.cells.insert(i, (key, value)),
        }
    }

    /// Insert or replace `key`. Errors if the packed page would overflow.
    pub fn upsert(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.upsert_unchecked(key, value);
        let mut scratch = Page::zeroed(0);
        self.pack_into(&mut scratch)?;
        Ok(())
    }

    /// True when this leaf packs into one page.
    pub fn fits(&self) -> bool {
        let mut scratch = Page::zeroed(0);
        self.pack_into(&mut scratch).is_ok()
    }

    /// Split at midpoint into `(left, right, separator_key)`.
    ///
    /// `separator_key` is the first key of `right` (copy separator for parents).
    pub fn split_half(mut self) -> Result<(LeafPage, LeafPage, Vec<u8>)> {
        if self.cells.len() < 2 {
            return Err(Error::Corrupt("cannot split leaf with < 2 cells"));
        }
        let mid = self.cells.len() / 2;
        let right_cells = self.cells.split_off(mid);
        let separator = right_cells[0].0.clone();
        let right = LeafPage {
            next: self.next,
            cells: right_cells,
        };
        let left = LeafPage {
            next: 0, // filled by caller with right page id
            cells: self.cells,
        };
        Ok((left, right, separator))
    }

    /// Pack this leaf into `page` (overwrites `page.data`).
    pub fn pack_into(&self, page: &mut Page) -> Result<()> {
        if self.cells.len() > u16::MAX as usize {
            return Err(Error::Corrupt("too many leaf cells"));
        }
        page.data.fill(0);
        page.data[0] = PAGE_TYPE_LEAF;
        page.data[2..4].copy_from_slice(&(self.cells.len() as u16).to_le_bytes());
        page.data[4..8].copy_from_slice(&self.next.to_le_bytes());

        let mut cursor = PAGE_USABLE;
        let mut pointers: Vec<u16> = Vec::with_capacity(self.cells.len());
        for (key, value) in self.cells.iter().rev() {
            let need = 4 + key.len() + 4 + value.len();
            let pointer_bytes = (pointers.len() + 1) * 2;
            if HEADER_LEN + pointer_bytes + need > cursor {
                return Err(Error::Unsupported("leaf page full"));
            }
            cursor -= need;
            let start = cursor;
            write_u32_at(page.data.as_mut_slice(), start, key.len() as u32);
            page.data[start + 4..start + 4 + key.len()].copy_from_slice(key);
            let value_at = start + 4 + key.len();
            write_u32_at(page.data.as_mut_slice(), value_at, value.len() as u32);
            page.data[value_at + 4..value_at + 4 + value.len()].copy_from_slice(value);
            pointers.push(start as u16);
        }
        pointers.reverse();
        page.data[8..10].copy_from_slice(&(cursor as u16).to_le_bytes());
        for (i, ptr) in pointers.iter().enumerate() {
            let at = HEADER_LEN + i * 2;
            page.data[at..at + 2].copy_from_slice(&ptr.to_le_bytes());
        }
        Ok(())
    }

    /// Decode a leaf from `page`.
    pub fn unpack(page: &Page) -> Result<Self> {
        if page.data[0] != PAGE_TYPE_LEAF {
            return Err(Error::Corrupt("not a leaf page"));
        }
        let count = u16::from_le_bytes(page.data[2..4].try_into().unwrap()) as usize;
        let next = PageId::from_le_bytes(page.data[4..8].try_into().unwrap());
        let mut cells = Vec::with_capacity(count);
        for i in 0..count {
            let at = HEADER_LEN + i * 2;
            if at + 2 > PAGE_USABLE {
                return Err(Error::Corrupt("leaf cell pointer OOB"));
            }
            let start = u16::from_le_bytes(page.data[at..at + 2].try_into().unwrap()) as usize;
            let (key, value) = read_kv_cell(page.data.as_slice(), start)?;
            cells.push((key, value));
        }
        for w in cells.windows(2) {
            if w[0].0 > w[1].0 {
                return Err(Error::Corrupt("leaf cells not sorted"));
            }
        }
        Ok(Self { next, cells })
    }
}

/// Internal node: `leftmost` child plus `(separator_key → right_child)` entries.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InternalPage {
    /// Child for keys less than the first separator.
    pub leftmost: PageId,
    /// Separators sorted ascending; child holds keys `>=` that separator and
    /// `<` the next separator (or unbounded on the right).
    pub entries: Vec<(Vec<u8>, PageId)>,
}

impl InternalPage {
    /// Empty internal page.
    pub fn new(leftmost: PageId) -> Self {
        Self {
            leftmost,
            entries: Vec::new(),
        }
    }

    /// Choose the child page for `key`.
    pub fn child_for(&self, key: &[u8]) -> PageId {
        match self
            .entries
            .binary_search_by(|(k, _)| k.as_slice().cmp(key))
        {
            Ok(i) => self.entries[i].1,
            Err(0) => self.leftmost,
            Err(i) => self.entries[i - 1].1,
        }
    }

    /// Insert separator after a child split. `left_child` is the page that was
    /// split; `right_child` is the new page; `separator` is the first key of right.
    pub fn insert_after_split(
        &mut self,
        left_child: PageId,
        separator: Vec<u8>,
        right_child: PageId,
    ) -> Result<()> {
        self.insert_after_split_unchecked(left_child, separator, right_child)?;
        let mut scratch = Page::zeroed(0);
        self.pack_into(&mut scratch)?;
        Ok(())
    }

    /// Like [`Self::insert_after_split`] but skips capacity check.
    pub fn insert_after_split_unchecked(
        &mut self,
        left_child: PageId,
        separator: Vec<u8>,
        right_child: PageId,
    ) -> Result<()> {
        if self.leftmost == left_child {
            self.entries.insert(0, (separator, right_child));
            return Ok(());
        }
        let idx = self
            .entries
            .iter()
            .position(|(_, child)| *child == left_child)
            .ok_or(Error::Corrupt("split child not found in parent"))?;
        self.entries.insert(idx + 1, (separator, right_child));
        Ok(())
    }

    /// Remove `child` from this internal node (used when an empty leaf is unlinked).
    ///
    /// When `child` is `leftmost`, the first separator is dropped and its child
    /// becomes the new leftmost.
    pub fn remove_child(&mut self, child: PageId) -> Result<()> {
        if self.leftmost == child {
            if self.entries.is_empty() {
                return Err(Error::Corrupt("cannot remove sole child from internal"));
            }
            let (_sep, new_left) = self.entries.remove(0);
            self.leftmost = new_left;
            return Ok(());
        }
        let idx = self
            .entries
            .iter()
            .position(|(_, c)| *c == child)
            .ok_or(Error::Corrupt("child not found in parent"))?;
        self.entries.remove(idx);
        Ok(())
    }

    /// True when this internal page packs into one page.
    pub fn fits(&self) -> bool {
        let mut scratch = Page::zeroed(0);
        self.pack_into(&mut scratch).is_ok()
    }

    /// Split at midpoint into `(left, right, promoted_separator)`.
    pub fn split_half(mut self) -> Result<(InternalPage, InternalPage, Vec<u8>)> {
        if self.entries.len() < 3 {
            return Err(Error::Corrupt("cannot split internal with < 3 entries"));
        }
        let mid = self.entries.len() / 2;
        let separator = self.entries[mid].0.clone();
        let right_leftmost = self.entries[mid].1;
        let right_entries = self.entries.split_off(mid + 1);
        self.entries.truncate(mid);
        let left = InternalPage {
            leftmost: self.leftmost,
            entries: self.entries,
        };
        let right = InternalPage {
            leftmost: right_leftmost,
            entries: right_entries,
        };
        Ok((left, right, separator))
    }

    /// Pack into `page`.
    pub fn pack_into(&self, page: &mut Page) -> Result<()> {
        if self.entries.len() > u16::MAX as usize {
            return Err(Error::Corrupt("too many internal entries"));
        }
        page.data.fill(0);
        page.data[0] = PAGE_TYPE_INTERNAL;
        page.data[2..4].copy_from_slice(&(self.entries.len() as u16).to_le_bytes());
        page.data[4..8].copy_from_slice(&self.leftmost.to_le_bytes());

        let mut cursor = PAGE_USABLE;
        let mut pointers: Vec<u16> = Vec::with_capacity(self.entries.len());
        for (key, child) in self.entries.iter().rev() {
            let need = 4 + key.len() + 4;
            let pointer_bytes = (pointers.len() + 1) * 2;
            if HEADER_LEN + pointer_bytes + need > cursor {
                return Err(Error::Unsupported("internal page full"));
            }
            cursor -= need;
            let start = cursor;
            write_u32_at(page.data.as_mut_slice(), start, key.len() as u32);
            page.data[start + 4..start + 4 + key.len()].copy_from_slice(key);
            let child_at = start + 4 + key.len();
            page.data[child_at..child_at + 4].copy_from_slice(&child.to_le_bytes());
            pointers.push(start as u16);
        }
        pointers.reverse();
        page.data[8..10].copy_from_slice(&(cursor as u16).to_le_bytes());
        for (i, ptr) in pointers.iter().enumerate() {
            let at = HEADER_LEN + i * 2;
            page.data[at..at + 2].copy_from_slice(&ptr.to_le_bytes());
        }
        Ok(())
    }

    /// Decode from `page`.
    pub fn unpack(page: &Page) -> Result<Self> {
        if page.data[0] != PAGE_TYPE_INTERNAL {
            return Err(Error::Corrupt("not an internal page"));
        }
        let count = u16::from_le_bytes(page.data[2..4].try_into().unwrap()) as usize;
        let leftmost = PageId::from_le_bytes(page.data[4..8].try_into().unwrap());
        let mut entries = Vec::with_capacity(count);
        for i in 0..count {
            let at = HEADER_LEN + i * 2;
            let start = u16::from_le_bytes(page.data[at..at + 2].try_into().unwrap()) as usize;
            let (key, child) = read_internal_cell(page.data.as_slice(), start)?;
            entries.push((key, child));
        }
        Ok(Self { leftmost, entries })
    }
}

/// Minimal B+Tree over a [`Pager`] (root page id stored in meta page).
#[derive(Debug)]
pub struct BTree<'a> {
    pager: &'a mut Pager,
    root: PageId,
}

impl<'a> BTree<'a> {
    /// B+Tree root page id from the meta page (`0` = not created yet).
    pub fn read_root(pager: &mut Pager) -> Result<PageId> {
        get_root(pager)
    }

    /// Create a new tree with an empty leaf root.
    pub fn create(pager: &'a mut Pager) -> Result<Self> {
        let root = pager.allocate_page()?;
        let leaf = LeafPage::new();
        let mut page = Page::zeroed(root);
        leaf.pack_into(&mut page)?;
        pager.write_page(&page)?;
        set_root(pager, root)?;
        Ok(Self { pager, root })
    }

    /// Open an existing tree from the pager meta root pointer.
    pub fn open(pager: &'a mut Pager) -> Result<Self> {
        let root = get_root(pager)?;
        if root == 0 {
            return Err(Error::Corrupt("btree root not set"));
        }
        Ok(Self { pager, root })
    }

    /// Fetch a value by key.
    pub fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let leaf_id = self.find_leaf(key)?;
        let page = self.pager.read_page(leaf_id)?;
        let leaf = LeafPage::unpack(&page)?;
        Ok(leaf.get(key).map(|v| v.to_vec()))
    }

    /// Delete `key` if present. Returns whether a value was removed.
    ///
    /// When a non-root leaf becomes empty it is unlinked from the sibling chain
    /// and removed from its parent; a root that collapses to one child shrinks
    /// tree height. Orphaned pages are not yet returned to a free list.
    pub fn delete(&mut self, key: &[u8]) -> Result<bool> {
        let (leaf_id, path) = self.find_leaf_path(key)?;
        let page = self.pager.read_page(leaf_id)?;
        let mut leaf = LeafPage::unpack(&page)?;
        if !leaf.remove(key) {
            return Ok(false);
        }
        if leaf.cells.is_empty() && leaf_id != self.root {
            self.unlink_empty_leaf(leaf_id, &leaf, &path)?;
        } else {
            let mut page = Page::zeroed(leaf_id);
            leaf.pack_into(&mut page)?;
            self.pager.write_page(&page)?;
        }
        Ok(true)
    }

    /// Unlink an empty non-root leaf and drop it from the parent.
    fn unlink_empty_leaf(
        &mut self,
        leaf_id: PageId,
        leaf: &LeafPage,
        path: &[PageId],
    ) -> Result<()> {
        if path.is_empty() {
            return Err(Error::Corrupt("empty non-root leaf missing parent path"));
        }
        if let Some(left_id) = self.left_sibling_leaf(leaf_id, path)? {
            let page = self.pager.read_page(left_id)?;
            let mut left = LeafPage::unpack(&page)?;
            left.next = leaf.next;
            let mut out = Page::zeroed(left_id);
            left.pack_into(&mut out)?;
            self.pager.write_page(&out)?;
        }

        let parent_id = path[path.len() - 1];
        let page = self.pager.read_page(parent_id)?;
        let mut parent = InternalPage::unpack(&page)?;
        parent.remove_child(leaf_id)?;

        if parent_id == self.root && parent.entries.is_empty() {
            self.root = parent.leftmost;
            set_root(self.pager, self.root)?;
            self.pager.free_page(leaf_id)?;
            self.pager.free_page(parent_id)?;
            return Ok(());
        }

        let mut out = Page::zeroed(parent_id);
        parent.pack_into(&mut out)?;
        self.pager.write_page(&out)?;
        self.pager.free_page(leaf_id)?;
        Ok(())
    }

    /// Left sibling leaf under the same parent, if any.
    fn left_sibling_leaf(&mut self, leaf_id: PageId, path: &[PageId]) -> Result<Option<PageId>> {
        let parent_id = path[path.len() - 1];
        let page = self.pager.read_page(parent_id)?;
        let parent = InternalPage::unpack(&page)?;
        if parent.leftmost == leaf_id {
            return Ok(None);
        }
        if parent.entries.first().map(|(_, c)| *c) == Some(leaf_id) {
            return Ok(Some(parent.leftmost));
        }
        for i in 1..parent.entries.len() {
            if parent.entries[i].1 == leaf_id {
                return Ok(Some(parent.entries[i - 1].1));
            }
        }
        Err(Error::Corrupt(
            "leaf not found in parent for sibling lookup",
        ))
    }

    /// Scan keys in `[start, end)` order (byte-wise). `end = None` means unbounded.
    ///
    /// Starts at the leaf for `start` (or the leftmost leaf when `start` is `None`)
    /// and follows `LeafPage::next` siblings.
    pub fn range(
        &mut self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        if let (Some(s), Some(e)) = (start, end) {
            if s >= e {
                return Ok(Vec::new());
            }
        }
        let mut leaf_id = match start {
            Some(key) => self.find_leaf(key)?,
            None => self.leftmost_leaf()?,
        };
        let mut out = Vec::new();
        loop {
            let page = self.pager.read_page(leaf_id)?;
            let leaf = LeafPage::unpack(&page)?;
            for (key, value) in &leaf.cells {
                if let Some(s) = start {
                    if key.as_slice() < s {
                        continue;
                    }
                }
                if let Some(e) = end {
                    if key.as_slice() >= e {
                        return Ok(out);
                    }
                }
                out.push((key.clone(), value.clone()));
            }
            if leaf.next == 0 {
                break;
            }
            leaf_id = leaf.next;
        }
        Ok(out)
    }

    /// Insert or replace a key/value (splits leaves/internals; grows height).
    pub fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        let (leaf_id, path) = self.find_leaf_path(key)?;
        let page = self.pager.read_page(leaf_id)?;
        let mut leaf = LeafPage::unpack(&page)?;
        leaf.upsert_unchecked(key.to_vec(), value.to_vec());
        if leaf.fits() {
            let mut page = Page::zeroed(leaf_id);
            leaf.pack_into(&mut page)?;
            self.pager.write_page(&page)?;
            return Ok(());
        }

        let (mut left, right, separator) = leaf.split_half()?;
        let right_id = self.pager.allocate_page()?;
        left.next = right_id;
        let mut left_page = Page::zeroed(leaf_id);
        left.pack_into(&mut left_page)?;
        self.pager.write_page(&left_page)?;
        let mut right_page = Page::zeroed(right_id);
        right.pack_into(&mut right_page)?;
        self.pager.write_page(&right_page)?;

        self.insert_into_ancestors(&path, leaf_id, separator, right_id)
    }

    fn insert_into_ancestors(
        &mut self,
        path: &[PageId],
        left_child: PageId,
        separator: Vec<u8>,
        right_child: PageId,
    ) -> Result<()> {
        if path.is_empty() {
            let new_root = self.pager.allocate_page()?;
            let mut internal = InternalPage::new(left_child);
            internal.entries.push((separator, right_child));
            let mut page = Page::zeroed(new_root);
            internal.pack_into(&mut page)?;
            self.pager.write_page(&page)?;
            self.root = new_root;
            set_root(self.pager, new_root)?;
            return Ok(());
        }

        let parent_id = path[path.len() - 1];
        let page = self.pager.read_page(parent_id)?;
        let mut parent = InternalPage::unpack(&page)?;
        parent.insert_after_split_unchecked(left_child, separator, right_child)?;
        if parent.fits() {
            let mut out = Page::zeroed(parent_id);
            parent.pack_into(&mut out)?;
            self.pager.write_page(&out)?;
            return Ok(());
        }

        let (left, right, promoted) = parent.split_half()?;
        let right_id = self.pager.allocate_page()?;
        let mut left_page = Page::zeroed(parent_id);
        left.pack_into(&mut left_page)?;
        self.pager.write_page(&left_page)?;
        let mut right_page = Page::zeroed(right_id);
        right.pack_into(&mut right_page)?;
        self.pager.write_page(&right_page)?;

        self.insert_into_ancestors(&path[..path.len() - 1], parent_id, promoted, right_id)
    }

    fn find_leaf(&mut self, key: &[u8]) -> Result<PageId> {
        Ok(self.find_leaf_path(key)?.0)
    }

    fn leftmost_leaf(&mut self) -> Result<PageId> {
        let mut id = self.root;
        loop {
            let page = self.pager.read_page(id)?;
            match page.data[0] {
                PAGE_TYPE_LEAF => return Ok(id),
                PAGE_TYPE_INTERNAL => {
                    let internal = InternalPage::unpack(&page)?;
                    id = internal.leftmost;
                }
                _ => return Err(Error::Corrupt("unknown btree page type")),
            }
        }
    }

    fn find_leaf_path(&mut self, key: &[u8]) -> Result<(PageId, Vec<PageId>)> {
        let mut path = Vec::new();
        let mut id = self.root;
        loop {
            let page = self.pager.read_page(id)?;
            match page.data[0] {
                PAGE_TYPE_LEAF => return Ok((id, path)),
                PAGE_TYPE_INTERNAL => {
                    path.push(id);
                    let internal = InternalPage::unpack(&page)?;
                    id = internal.child_for(key);
                }
                _ => return Err(Error::Corrupt("unknown btree page type")),
            }
        }
    }
}

fn get_root(pager: &mut Pager) -> Result<PageId> {
    let meta = pager.read_page(0)?;
    if meta.data[..5] != *PAGER_MAGIC {
        return Err(Error::Corrupt("unknown pager magic"));
    }
    Ok(PageId::from_le_bytes(
        meta.data[META_BTREE_ROOT_OFFSET..META_BTREE_ROOT_OFFSET + 4]
            .try_into()
            .unwrap(),
    ))
}

fn set_root(pager: &mut Pager, root: PageId) -> Result<()> {
    let mut meta = pager.read_page(0)?;
    meta.data[META_BTREE_ROOT_OFFSET..META_BTREE_ROOT_OFFSET + 4]
        .copy_from_slice(&root.to_le_bytes());
    pager.write_page(&meta)
}

fn write_u32_at(buf: &mut [u8], at: usize, value: u32) {
    buf[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

fn read_kv_cell(buf: &[u8], start: usize) -> Result<(Vec<u8>, Vec<u8>)> {
    if start + 4 > buf.len() {
        return Err(Error::Corrupt("leaf cell truncated"));
    }
    let key_len = u32::from_le_bytes(buf[start..start + 4].try_into().unwrap()) as usize;
    let key_at = start + 4;
    if key_at + key_len + 4 > buf.len() {
        return Err(Error::Corrupt("leaf key truncated"));
    }
    let key = buf[key_at..key_at + key_len].to_vec();
    let value_at = key_at + key_len;
    let value_len = u32::from_le_bytes(buf[value_at..value_at + 4].try_into().unwrap()) as usize;
    let value_body = value_at + 4;
    if value_body + value_len > buf.len() {
        return Err(Error::Corrupt("leaf value truncated"));
    }
    let value = buf[value_body..value_body + value_len].to_vec();
    Ok((key, value))
}

fn read_internal_cell(buf: &[u8], start: usize) -> Result<(Vec<u8>, PageId)> {
    if start + 4 > buf.len() {
        return Err(Error::Corrupt("internal cell truncated"));
    }
    let key_len = u32::from_le_bytes(buf[start..start + 4].try_into().unwrap()) as usize;
    let key_at = start + 4;
    if key_at + key_len + 4 > buf.len() {
        return Err(Error::Corrupt("internal key truncated"));
    }
    let key = buf[key_at..key_at + key_len].to_vec();
    let child_at = key_at + key_len;
    let child = PageId::from_le_bytes(buf[child_at..child_at + 4].try_into().unwrap());
    Ok((key, child))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(label: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("yydb-btree-{label}-{nonce}.tmp"))
    }

    #[test]
    fn leaf_pack_roundtrip_and_search() {
        let mut leaf = LeafPage::new();
        leaf.upsert(b"a".to_vec(), b"1".to_vec()).unwrap();
        leaf.upsert(b"c".to_vec(), b"3".to_vec()).unwrap();
        leaf.upsert(b"b".to_vec(), b"2".to_vec()).unwrap();
        assert_eq!(leaf.get(b"b"), Some(b"2".as_slice()));

        let mut page = Page::zeroed(1);
        leaf.pack_into(&mut page).unwrap();
        let decoded = LeafPage::unpack(&page).unwrap();
        assert_eq!(decoded.cells.len(), 3);
        assert_eq!(decoded.get(b"a"), Some(b"1".as_slice()));
        assert_eq!(decoded.get(b"c"), Some(b"3".as_slice()));
    }

    #[test]
    fn leaf_split_half() {
        let mut leaf = LeafPage::new();
        for i in 0..10u8 {
            leaf.upsert_unchecked(vec![i], vec![i]);
        }
        let (left, right, sep) = leaf.split_half().unwrap();
        assert_eq!(left.cells.len(), 5);
        assert_eq!(right.cells.len(), 5);
        assert_eq!(sep, vec![5]);
        assert!(left.cells.last().unwrap().0 < right.cells.first().unwrap().0);
    }

    #[test]
    fn btree_put_get_and_split_root() {
        let path = temp_path("split");
        let mut pager = Pager::create(&path).unwrap();
        let mut tree = BTree::create(&mut pager).unwrap();

        // Fill with large values so the root leaf must split.
        let payload = vec![0xAB; 800];
        for i in 0..8u8 {
            tree.put(&[i], &payload).unwrap();
        }
        for i in 0..8u8 {
            assert_eq!(tree.get(&[i]).unwrap().as_deref(), Some(payload.as_slice()));
        }

        // Root should now be internal.
        let root = tree.root;
        let page = tree.pager.read_page(root).unwrap();
        assert_eq!(page.data[0], PAGE_TYPE_INTERNAL);

        drop(tree);
        let mut pager = Pager::open(&path).unwrap();
        let mut tree = BTree::open(&mut pager).unwrap();
        assert_eq!(tree.get(&[3]).unwrap().as_deref(), Some(payload.as_slice()));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn btree_grows_past_single_internal() {
        let path = temp_path("multilevel");
        let mut pager = Pager::create(&path).unwrap();
        let mut tree = BTree::create(&mut pager).unwrap();
        // Small values but many keys → repeated leaf + internal splits.
        for i in 0..400u16 {
            let key = i.to_be_bytes();
            tree.put(&key, &key).unwrap();
        }
        for i in 0..400u16 {
            let key = i.to_be_bytes();
            assert_eq!(tree.get(&key).unwrap().as_deref(), Some(key.as_slice()));
        }
        // Walk height: root internal, at least one child internal or many leaf children.
        let root = tree.root;
        let page = tree.pager.read_page(root).unwrap();
        assert_eq!(page.data[0], PAGE_TYPE_INTERNAL);
        let internal = InternalPage::unpack(&page).unwrap();
        assert!(internal.entries.len() >= 1);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn btree_delete_and_range() {
        let path = temp_path("del-range");
        let mut pager = Pager::create(&path).unwrap();
        let mut tree = BTree::create(&mut pager).unwrap();
        for i in 0..50u16 {
            let key = i.to_be_bytes();
            tree.put(&key, &key).unwrap();
        }
        assert!(tree.delete(&10u16.to_be_bytes()).unwrap());
        assert!(!tree.delete(&10u16.to_be_bytes()).unwrap());
        assert_eq!(tree.get(&10u16.to_be_bytes()).unwrap(), None);

        let ranged = tree
            .range(Some(&5u16.to_be_bytes()), Some(&9u16.to_be_bytes()))
            .unwrap();
        let keys: Vec<u16> = ranged
            .iter()
            .map(|(k, _)| u16::from_be_bytes(k.as_slice().try_into().unwrap()))
            .collect();
        assert_eq!(keys, vec![5, 6, 7, 8]);

        let all = tree.range(None, None).unwrap();
        assert_eq!(all.len(), 49);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn btree_delete_unlinks_empty_leaf_and_shrinks_root() {
        let path = temp_path("del-merge");
        let mut pager = Pager::create(&path).unwrap();
        let mut tree = BTree::create(&mut pager).unwrap();
        let payload = vec![0xCD; 800];
        for i in 0..8u8 {
            tree.put(&[i], &payload).unwrap();
        }
        let root_after_split = tree.root;
        let page = tree.pager.read_page(root_after_split).unwrap();
        assert_eq!(page.data[0], PAGE_TYPE_INTERNAL);
        let internal = InternalPage::unpack(&page).unwrap();
        assert!(!internal.entries.is_empty());

        // Delete every key; empty leaves unlink and height should collapse.
        for i in 0..8u8 {
            assert!(tree.delete(&[i]).unwrap());
        }
        assert_eq!(tree.range(None, None).unwrap().len(), 0);
        let root = tree.root;
        let page = tree.pager.read_page(root).unwrap();
        assert_eq!(page.data[0], PAGE_TYPE_LEAF);
        let leaf = LeafPage::unpack(&page).unwrap();
        assert!(leaf.cells.is_empty());
        let _ = std::fs::remove_file(&path);
    }
}
