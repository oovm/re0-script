use std::{num::ParseIntError, ops::Range};

use url::Url;

pub type LifeResult<T> = std::result::Result<T, LifeError>;

#[derive(Clone, Debug)]
pub struct LifeError {
    kind: Box<LifeErrorKind>,
}

#[derive(Clone, Debug)]
pub enum LifeErrorKind {
    RuntimeError { message: String },
    SyntaxError { message: String, file: Option<Url>, span: Range<usize> },
    DuplicateError { message: String, old: (Option<Url>, Range<usize>), new: (Option<Url>, Range<usize>) },
}

impl LifeError {
    pub fn kind(&self) -> &LifeErrorKind {
        &self.kind
    }
}

impl From<LifeErrorKind> for LifeError {
    fn from(value: LifeErrorKind) -> Self {
        Self { kind: Box::new(value) }
    }
}

impl From<ParseIntError> for LifeError {
    fn from(value: ParseIntError) -> Self {
        LifeErrorKind::SyntaxError { message: value.to_string(), file: None, span: Default::default() }.into()
    }
}
