use super::*;

#[derive(Clone, Debug)]
pub struct TalentManager {
    /// 从一开始的属性
    counter: NonZeroUsize,
    talents: BTreeMap<NonZeroUsize, TalentItem>,
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
        TalentManager { counter: unsafe { NonZeroUsize::new_unchecked(1) }, talents: Default::default(), buffer: vec![] }
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
                if self.talents.contains_key(&s) {
                    Err(LifeErrorKind::DuplicateError {
                        message: "Duplicate talent id".to_string(),
                        old: (self.talents.get(&s).unwrap().file.clone(), self.talents.get(&s).unwrap().span.clone()),
                        new: (item.file.clone(), item.span.clone()),
                    })?
                }
                else {
                    self.talents.insert(s, item);
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
                if self.talents.contains_key(&self.counter) {
                    self.counter = self.counter.saturating_add(1);
                    continue;
                }
                else {
                    item.id = Some(self.counter);
                    self.talents.insert(self.counter, item);
                    break;
                }
            }
        }
    }
}

impl TalentItem {}
