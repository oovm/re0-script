//! Catalog pages for the paged `.yydb` layout (`YYDB\x02`).
//!
//! Stores schema + [`crate::schema_catalog::SchemaCatalog`] in one or more
//! catalog pages referenced from the meta page (`catalog_root`).

use crate::pager::{Page, PageId, Pager, PAGE_USABLE};
use crate::schema_catalog;
use yydb_types::{Error, Result, SchemaVersion};

/// Page type byte for catalog payload pages.
pub const PAGE_TYPE_CATALOG: u8 = 0x0c;

const HEADER_LEN: usize = 24;

/// Serialize catalog state into pager pages and return the root page id.
pub fn write_catalog(pager: &mut Pager, state: &CatalogState) -> Result<PageId> {
    let payload = encode_payload(state)?;
    write_payload_chain(pager, &payload)
}

/// Load catalog state from `root` (`0` ⇒ empty).
pub fn read_catalog(pager: &mut Pager, root: PageId) -> Result<CatalogState> {
    if root == 0 {
        return Ok(CatalogState::default());
    }
    let payload = read_payload_chain(pager, root)?;
    // Empty page (pre-marker) and explicit `[0]` both mean no schema.
    if payload.is_empty() {
        return Ok(CatalogState::default());
    }
    decode_payload(&payload)
}

/// Allocate an empty catalog page and wire `meta.catalog_root`.
pub fn init_empty(pager: &mut Pager) -> Result<PageId> {
    let root = pager.allocate_page()?;
    pager.write_page(&blank_catalog_page(root))?;
    crate::pager::write_catalog_root(pager, root)?;
    Ok(root)
}

/// In-memory catalog payload carried on paged catalog pages.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CatalogState {
    /// VOS schema when present.
    pub schema: Option<SchemaVersion>,
    /// Field-identity catalog when present.
    pub schema_catalog: Option<schema_catalog::SchemaCatalog>,
}

impl CatalogState {
    /// Build from schema + catalog parts (records live outside paged catalog pages).
    pub fn new(
        schema: Option<SchemaVersion>,
        schema_catalog: Option<schema_catalog::SchemaCatalog>,
    ) -> Self {
        Self {
            schema,
            schema_catalog,
        }
    }
}

fn encode_payload(state: &CatalogState) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    match &state.schema {
        Some(schema) => {
            bytes.push(1);
            bytes.extend(schema.version.to_le_bytes());
            write_bytes(&mut bytes, schema.document.as_bytes())?;
            match &state.schema_catalog {
                Some(catalog) => {
                    bytes.push(2);
                    write_bytes(&mut bytes, &catalog.encode()?)?;
                }
                None => bytes.push(0),
            }
        }
        None => bytes.push(0),
    }
    Ok(bytes)
}

fn decode_payload(bytes: &[u8]) -> Result<CatalogState> {
    let mut cursor = 0;
    let schema = match take_byte(bytes, &mut cursor)? {
        0 => None,
        1 => {
            let version = read_u32(bytes, &mut cursor)?;
            let document = String::from_utf8(read_bytes(bytes, &mut cursor)?)
                .map_err(|_| Error::Corrupt("catalog document is not UTF-8"))?;
            Some(SchemaVersion { version, document })
        }
        _ => return Err(Error::Corrupt("unknown catalog schema marker")),
    };
    let schema_catalog = if schema.is_some() {
        match take_byte(bytes, &mut cursor)? {
            0 => None,
            2 => Some(schema_catalog::SchemaCatalog::decode(&read_bytes(
                bytes,
                &mut cursor,
            )?)?),
            _ => return Err(Error::Corrupt("unknown catalog blob marker")),
        }
    } else {
        None
    };
    if cursor != bytes.len() {
        return Err(Error::Corrupt("trailing catalog bytes"));
    }
    Ok(CatalogState {
        schema,
        schema_catalog,
    })
}

fn write_payload_chain(pager: &mut Pager, payload: &[u8]) -> Result<PageId> {
    let chunk_cap = PAGE_USABLE.saturating_sub(HEADER_LEN);
    if chunk_cap == 0 {
        return Err(Error::Corrupt("catalog page too small"));
    }
    if payload.is_empty() {
        return init_empty(pager);
    }

    let chunks: Vec<&[u8]> = payload.chunks(chunk_cap).collect();
    let mut ids = Vec::with_capacity(chunks.len());
    for _ in 0..chunks.len() {
        ids.push(pager.allocate_page()?);
    }
    let root = ids[0];
    for (i, (id, chunk)) in ids.iter().zip(chunks.iter()).enumerate() {
        let next = if i + 1 < ids.len() { ids[i + 1] } else { 0 };
        let page = pack_catalog_page(*id, chunk, next != 0, next, payload.len() as u32);
        pager.write_page(&page)?;
    }
    crate::pager::write_catalog_root(pager, root)?;
    Ok(root)
}

fn read_payload_chain(pager: &mut Pager, mut page_id: PageId) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut total_len: Option<u32> = None;
    loop {
        let page = pager.read_page(page_id)?;
        let (chunk, has_next, next, total) = unpack_catalog_page(&page)?;
        if let Some(expected) = total_len {
            if expected != total {
                return Err(Error::Corrupt("catalog total length mismatch"));
            }
        } else {
            total_len = Some(total);
        }
        out.extend_from_slice(&chunk);
        if !has_next {
            break;
        }
        if next == 0 {
            return Err(Error::Corrupt("catalog chain missing next page"));
        }
        page_id = next;
    }
    if let Some(expected) = total_len {
        if out.len() != expected as usize {
            return Err(Error::Corrupt("catalog payload length mismatch"));
        }
    }
    Ok(out)
}

fn blank_catalog_page(id: PageId) -> Page {
    // Default encoded catalog: schema marker `0` (no schema).
    pack_catalog_page(id, &[0], false, 0, 1)
}

fn pack_catalog_page(
    id: PageId,
    chunk: &[u8],
    has_next: bool,
    next: PageId,
    total_len: u32,
) -> Page {
    let mut page = Page::zeroed(id);
    page.data[0] = PAGE_TYPE_CATALOG;
    page.data[1] = u8::from(has_next);
    page.data[4..8].copy_from_slice(&next.to_le_bytes());
    page.data[8..12].copy_from_slice(&total_len.to_le_bytes());
    let chunk_len = u32::try_from(chunk.len()).expect("chunk fits u32");
    page.data[12..16].copy_from_slice(&chunk_len.to_le_bytes());
    let end = HEADER_LEN + chunk.len();
    if end > PAGE_USABLE {
        panic!("catalog chunk overflow");
    }
    page.data[HEADER_LEN..end].copy_from_slice(chunk);
    page
}

fn unpack_catalog_page(page: &Page) -> Result<(Vec<u8>, bool, PageId, u32)> {
    if page.data[0] != PAGE_TYPE_CATALOG {
        return Err(Error::Corrupt("not a catalog page"));
    }
    let has_next = page.data[1] != 0;
    let next = u32::from_le_bytes(page.data[4..8].try_into().unwrap());
    let total = u32::from_le_bytes(page.data[8..12].try_into().unwrap());
    let chunk_len = u32::from_le_bytes(page.data[12..16].try_into().unwrap()) as usize;
    let end = HEADER_LEN
        .checked_add(chunk_len)
        .ok_or(Error::Corrupt("catalog chunk length overflow"))?;
    if end > PAGE_USABLE {
        return Err(Error::Corrupt("catalog chunk out of range"));
    }
    Ok((page.data[HEADER_LEN..end].to_vec(), has_next, next, total))
}

fn write_bytes(out: &mut Vec<u8>, value: &[u8]) -> Result<()> {
    let len = u32::try_from(value.len()).map_err(|_| Error::Corrupt("catalog blob too large"))?;
    out.extend(len.to_le_bytes());
    out.extend(value);
    Ok(())
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32> {
    let end = cursor
        .checked_add(4)
        .ok_or(Error::Corrupt("catalog cursor overflow"))?;
    let raw = bytes
        .get(*cursor..end)
        .ok_or(Error::Corrupt("catalog truncated u32"))?;
    *cursor = end;
    Ok(u32::from_le_bytes(raw.try_into().unwrap()))
}

fn read_bytes(bytes: &[u8], cursor: &mut usize) -> Result<Vec<u8>> {
    let len =
        usize::try_from(read_u32(bytes, cursor)?).map_err(|_| Error::Corrupt("catalog length"))?;
    let end = cursor
        .checked_add(len)
        .ok_or(Error::Corrupt("catalog cursor overflow"))?;
    let value = bytes
        .get(*cursor..end)
        .ok_or(Error::Corrupt("catalog truncated blob"))?
        .to_vec();
    *cursor = end;
    Ok(value)
}

fn take_byte(bytes: &[u8], cursor: &mut usize) -> Result<u8> {
    let value = *bytes
        .get(*cursor)
        .ok_or(Error::Corrupt("catalog truncated marker"))?;
    *cursor += 1;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(label: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("yydb-catalog-{label}-{nonce}.yydb"))
    }

    #[test]
    fn catalog_roundtrip_empty_and_schema() {
        let path = temp_path("rt");
        let mut pager = Pager::create(&path).unwrap();
        init_empty(&mut pager).unwrap();
        let root = crate::pager::read_catalog_root(&mut pager).unwrap();
        let empty = read_catalog(&mut pager, root).unwrap();
        assert!(empty.schema.is_none());

        let doc = "table User { @@user_id: uuid, @user_name: utf8 }";
        schema::validate_document(doc).unwrap();
        let vos = schema::parse_document(doc).unwrap();
        let catalog = schema_catalog::SchemaCatalog::from_document(&vos).unwrap();
        let state = CatalogState {
            schema: Some(SchemaVersion {
                version: 1,
                document: doc.into(),
            }),
            schema_catalog: Some(catalog),
        };
        write_catalog(&mut pager, &state).unwrap();
        drop(pager);

        let mut reopened = Pager::open(&path).unwrap();
        let root = crate::pager::read_catalog_root(&mut reopened).unwrap();
        let got = read_catalog(&mut reopened, root).unwrap();
        assert_eq!(got.schema.as_ref().unwrap().document, doc);
        assert!(got.schema_catalog.is_some());
        let _ = std::fs::remove_file(path);
    }
}
