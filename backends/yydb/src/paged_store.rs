//! Primary KV records on B+Tree pages for paged `.yydb` databases.
//!
//! Keys are UTF-8 strings (same namespace as snapshot-era `State::records`),
//! including row payloads and `idx/…` unique-index entries. Values are opaque
//! byte blobs.

use std::collections::{BTreeMap, BTreeSet};

use crate::btree::BTree;
use crate::pager::Pager;
use yydb_types::{Error, Result};

/// Load all committed KV records from the pager btree (`btree_root == 0` ⇒ empty).
pub fn load_records(pager: &mut Pager) -> Result<BTreeMap<String, Vec<u8>>> {
    if BTree::read_root(pager)? == 0 {
        return Ok(BTreeMap::new());
    }
    let mut tree = BTree::open(pager)?;
    let pairs = tree.range(None, None)?;
    let mut records = BTreeMap::new();
    for (key, value) in pairs {
        let key = String::from_utf8(key).map_err(|_| Error::Corrupt("record key is not utf-8"))?;
        records.insert(key, value);
    }
    Ok(records)
}

/// Persist `records` to the pager btree (creates the tree on first non-empty write).
pub fn store_records(pager: &mut Pager, records: &BTreeMap<String, Vec<u8>>) -> Result<()> {
    let root = BTree::read_root(pager)?;
    if root == 0 {
        if records.is_empty() {
            return Ok(());
        }
        let mut tree = BTree::create(pager)?;
        for (key, value) in records {
            tree.put(key.as_bytes(), value)?;
        }
        return Ok(());
    }

    let mut tree = BTree::open(pager)?;
    let existing: BTreeSet<Vec<u8>> = tree
        .range(None, None)?
        .into_iter()
        .map(|(k, _)| k)
        .collect();
    let desired: BTreeSet<Vec<u8>> = records.keys().map(|k| k.as_bytes().to_vec()).collect();
    for key in existing.difference(&desired) {
        tree.delete(key)?;
    }
    for (key, value) in records {
        tree.put(key.as_bytes(), value)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog_page;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(label: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("yydb-paged-store-{label}-{nonce}.yydb"))
    }

    #[test]
    fn kv_roundtrip_on_pager_btree() {
        let path = temp_path("kv");
        let mut pager = Pager::create(&path).unwrap();
        catalog_page::init_empty(&mut pager).unwrap();

        let mut records = BTreeMap::new();
        records.insert("project/meta".into(), b"Spark".to_vec());
        records.insert("row/Note/a".into(), b"payload".to_vec());
        store_records(&mut pager, &records).unwrap();
        drop(pager);

        let mut reopened = Pager::open(&path).unwrap();
        let got = load_records(&mut reopened).unwrap();
        assert_eq!(got, records);
        let _ = std::fs::remove_file(path);
    }
}
