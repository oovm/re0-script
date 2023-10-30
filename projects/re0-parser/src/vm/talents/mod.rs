use super::*;

#[derive(Clone, Debug)]
pub struct TalentManager {
    /// 从一开始的属性
    property_id: NonZeroUsize,
    properties: BTreeMap<NonZeroUsize, TalentItem>,
    // 暂时没分配到 id 的属性
    buffer: Vec<TalentItem>,
}

#[derive(Clone, Debug)]
pub struct TalentItem {
    pub name: String,
    pub id: Option<NonZeroUsize>,
    pub file: Option<Url>,
    pub span: Range<usize>,
}

impl Default for TalentManager {
    fn default() -> Self {
        TalentManager { property_id: unsafe { NonZeroUsize::new_unchecked(1) }, properties: Default::default(), buffer: vec![] }
    }
}

impl Default for TalentItem {
    fn default() -> Self {
        Self { name: "".to_string(), id: None, file: None, span: Default::default() }
    }
}

impl TalentManager {
    pub fn insert(&mut self, item: TalentItem) -> Result<(), LifeError> {
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
                if self.properties.contains_key(&self.property_id) {
                    self.property_id = self.property_id.saturating_add(1);
                    continue;
                }
                else {
                    item.id = Some(self.property_id);
                    self.properties.insert(self.property_id, item);
                    break;
                }
            }
        }
    }
}

impl TalentItem {}
