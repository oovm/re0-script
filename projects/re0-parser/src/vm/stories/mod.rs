use super::*;
use std::fmt::{Debug, Formatter};

#[derive(Clone, Debug)]
pub struct StoryManager {
    /// 计数器
    indexer: NonZeroUsize,
    stories: BTreeMap<NonZeroUsize, StoryItem>,
}

#[derive(Clone)]
pub struct StoryItem {
    pub id: Identifier,
    /// 属性 ID, 用于快速查询
    pub index: Option<NonZeroUsize>,
    /// 描述文本
    pub text: Vec<String>,
}

impl Debug for StoryItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let w = &mut f.debug_struct("StoryItem");
        w.field("name", &self.id.name);
        if let Some(s) = self.index {
            w.field("id", &s);
        }
        w.field("span", &self.id.display());
        w.finish()
    }
}

impl Default for StoryManager {
    fn default() -> Self {
        StoryManager { indexer: unsafe { NonZeroUsize::new_unchecked(1) }, stories: Default::default() }
    }
}

impl StoryManager {
    pub fn insert(&mut self, mut item: StoryItem) -> Result<(), LifeError> {
        let index = item.index.unwrap_or(self.indexer);
        match self.stories.get(&index) {
            Some(old) => Err(LifeErrorKind::DuplicateError {
                message: "Duplicate story id".to_string(),
                old: (old.id.file.clone(), old.id.span.clone()),
                new: (item.id.file, item.id.span),
            })?,
            None => {
                item.index = Some(index);
                self.stories.insert(index, item);
                self.indexer = index.saturating_add(1);
            }
        }
        Ok(())
    }
}

impl StoryItem {
    pub fn new(id: Identifier) -> Self {
        Self { id, index: None, text: vec![] }
    }
}
