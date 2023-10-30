use super::*;
use std::fmt::{Debug, Formatter};

#[derive(Clone, Debug)]
pub struct PropertyManager {
    /// 计数器
    indexer: NonZeroUsize,
    properties: BTreeMap<NonZeroUsize, PropertyItem>,
}

#[derive(Clone)]
pub struct PropertyItem {
    pub id: Identifier,
    /// 属性 ID, 用于快速查询
    pub index: Option<NonZeroUsize>,
    /// 描述文本
    pub text: Vec<String>,
}

impl Debug for PropertyItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let w = &mut f.debug_struct("PropertyItem");
        w.field("name", &self.id.name);
        if let Some(s) = self.index {
            w.field("id", &s);
        }
        w.field("span", &self.id.display());
        w.finish()
    }
}

impl Default for PropertyManager {
    fn default() -> Self {
        PropertyManager { indexer: unsafe { NonZeroUsize::new_unchecked(1) }, properties: Default::default() }
    }
}

impl PropertyManager {
    pub fn insert(&mut self, mut item: PropertyItem) -> Result<(), LifeError> {
        let index = item.index.unwrap_or(self.indexer);
        match self.properties.get(&index) {
            Some(old) => Err(LifeErrorKind::DuplicateError {
                message: "Duplicate property id".to_string(),
                old: (old.id.file.clone(), old.id.span.clone()),
                new: (item.id.file, item.id.span),
            })?,
            None => {
                item.index = Some(index);
                self.properties.insert(index, item);
                self.indexer = index.saturating_add(1);
            }
        }
        Ok(())
    }
}

impl PropertyItem {
    pub fn new(id: Identifier) -> Self {
        Self { id: id, index: None, text: vec![] }
    }
}
