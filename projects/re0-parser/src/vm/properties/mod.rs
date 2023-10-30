use super::*;
use crate::helper::PositionDisplay;
use std::fmt::{Debug, Formatter};

#[derive(Clone, Debug)]
pub struct PropertyManager {
    /// 计数器
    counter: NonZeroUsize,
    properties: BTreeMap<NonZeroUsize, PropertyItem>,
    // 暂时没分配到 id 的属性
    buffer: Vec<PropertyItem>,
}

#[derive(Clone)]
pub struct PropertyItem {
    /// 属性名字, 用于动态查询
    pub name: String,
    /// 属性 ID, 用于快速查询
    pub id: Option<NonZeroUsize>,
    /// 描述文本
    pub text: Vec<String>,
    /// 定义所在文件
    pub file: Option<Url>,
    /// 定义所在位置
    pub span: Range<usize>,
}

impl Debug for PropertyItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let w = &mut f.debug_struct("PropertyItem");
        w.field("name", &self.name);
        if let Some(s) = self.id {
            w.field("id", &s);
        }
        let span = PositionDisplay::new(&self.file, &self.span);
        w.field("span", &span);
        w.finish()
    }
}

impl Default for PropertyManager {
    fn default() -> Self {
        PropertyManager { counter: unsafe { NonZeroUsize::new_unchecked(1) }, properties: Default::default(), buffer: vec![] }
    }
}

impl Default for PropertyItem {
    fn default() -> Self {
        Self { name: "".to_string(), id: None, text: vec![], file: None, span: Default::default() }
    }
}

impl PropertyManager {
    pub fn insert(&mut self, item: PropertyItem) -> Result<(), LifeError> {
        match item.id {
            Some(s) => {
                if self.properties.contains_key(&s) {
                    Err(LifeErrorKind::DuplicateError {
                        message: "Duplicate property id".to_string(),
                        old: (self.properties.get(&s).unwrap().file.clone(), self.properties.get(&s).unwrap().span.clone()),
                        new: (item.file.clone(), item.span.clone()),
                    })?
                }
                else {
                    self.properties.insert(s, item);
                }
            }
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
                    item.id = Some(self.counter);
                    self.properties.insert(self.counter, item);
                    break;
                }
            }
        }
    }
}

impl PropertyItem {}
