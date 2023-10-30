use super::*;
use std::fmt::{Debug, Formatter};

#[derive(Clone, Debug)]
pub struct PropertyManager {
    /// 计数器
    counter: NonZeroUsize,
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
        PropertyManager { counter: unsafe { NonZeroUsize::new_unchecked(1) }, properties: Default::default() }
    }
}

impl PropertyManager {
    pub fn insert(&mut self, item: PropertyItem) -> Result<(), LifeError> {
        match item.index {
            Some(s) => match self.properties.get(&s) {
                Some(old) => Err(LifeErrorKind::DuplicateError {
                    message: "Duplicate property id".to_string(),
                    old: (old.id.file.clone(), old.id.span.clone()),
                    new: (item.id.file, item.id.span),
                })?,
                None => {
                    self.properties.insert(s, item);
                }
            },
            None => self.buffer.push(item),
        }
        Ok(())
    }

    pub fn finalize(&mut self) {
        for mut item in self.buffer.drain(..) {
            // 不断尝试插入到不存在的编号
            loop {
                if self.properties.contains_key(&self.counter) {
                    self.counter = self.counter.saturating_add(1);
                    continue;
                }
                else {
                    item.index = Some(self.counter);
                    self.properties.insert(self.counter, item);
                    break;
                }
            }
        }
    }
}

impl PropertyItem {
    pub fn new(id: Identifier) -> Self {
        Self { id: id, index: None, text: vec![] }
    }
}
