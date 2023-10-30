use super::*;
use crate::helper::PositionDisplay;

#[derive(Clone, Debug)]
pub struct Identifier {
    /// 属性名字, 用于动态查询
    pub name: String,
    /// 定义所在文件
    pub file: Option<Url>,
    /// 定义所在位置
    pub span: Range<usize>,
}

impl Identifier {
    pub(crate) fn display(&self) -> PositionDisplay {
        PositionDisplay::new(&self.file, &self.span)
    }
}
