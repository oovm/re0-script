use std::{num::ParseIntError, ops::Range};

use crate::codegen::LifeRestartRule;
use url::Url;
use yggdrasil_rt::YggdrasilError;

/// The result type of life restart
pub type LifeResult<T> = std::result::Result<T, LifeError>;

/// Error types for life restart
#[derive(Clone, Debug)]
pub struct LifeError {
    kind: Box<LifeErrorKind>,
}

/// The enumerate type of all errors
#[derive(Clone, Debug)]
pub enum LifeErrorKind {
    RuntimeError { message: String },
    SyntaxError { message: String, file: Option<Url>, span: Range<usize> },
    DuplicateError { message: String, old: (Option<Url>, Range<usize>), new: (Option<Url>, Range<usize>) },
}

impl LifeError {
    /// Get the kind of the error
    pub fn kind(&self) -> &LifeErrorKind {
        &self.kind
    }
}

impl From<LifeErrorKind> for LifeError {
    fn from(value: LifeErrorKind) -> Self {
        Self { kind: Box::new(value) }
    }
}
impl From<std::io::Error> for LifeError {
    fn from(value: std::io::Error) -> Self {
        LifeErrorKind::RuntimeError { message: value.to_string() }.into()
    }
}

impl From<()> for LifeError {
    #[track_caller]
    fn from(_: ()) -> Self {
        LifeErrorKind::RuntimeError { message: "void exception".to_string() }.into()
    }
}

impl From<ParseIntError> for LifeError {
    fn from(value: ParseIntError) -> Self {
        LifeErrorKind::SyntaxError { message: value.to_string(), file: None, span: Default::default() }.into()
    }
}
impl From<YggdrasilError<LifeRestartRule>> for LifeError {
    fn from(value: YggdrasilError<LifeRestartRule>) -> Self {
        LifeErrorKind::SyntaxError { message: value.variant.to_string(), file: None, span: Default::default() }.into()
    }
}
