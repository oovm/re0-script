use super::*;

#[derive(Clone, Debug)]
pub struct PropertyManager {
    /// 从一开始的属性
    property_id: NonZeroUsize,
    properties: BTreeMap<NonZeroUsize, PropertyItem>,
    // 暂时没分配到 id 的属性
    buffer: Vec<PropertyItem>,
}

#[derive(Copy, Clone, Debug)]
pub struct PropertyItem {
    id: Option<NonZeroUsize>,
}

impl Default for PropertyManager {
    fn default() -> Self {
        PropertyManager {
            property_id: unsafe { NonZeroUsize::new_unchecked(1) },
            properties: Default::default(),
            buffer: vec![],
        }
    }
}

impl PropertyManager {
    pub fn finalize(&mut self) {
        for item in self.buffer.drain(..) {
            // 不断尝试插入到不存在的编号
        }
    }
}
