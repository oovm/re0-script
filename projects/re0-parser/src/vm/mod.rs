use crate::codegen::{LifeRestartParser, LifeRestartRule, PropertyItemNode, RootNode, StatementNode};
use std::{collections::BTreeMap, num::NonZeroUsize, ops::Range, str::FromStr};
use yggdrasil_rt::YggdrasilParser;

use crate::errors::LifeErrorKind;
use url::Url;

pub use self::properties::{PropertyItem, PropertyManager};
use crate::{vm::talents::TalentManager, LifeError};

mod parser;
mod properties;
mod talents;

#[derive(Clone, Debug, Default)]
pub struct LifeVM {
    property: PropertyManager,
    talent: TalentManager,
}
