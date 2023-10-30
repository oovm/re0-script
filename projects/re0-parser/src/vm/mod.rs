use crate::codegen::{LifeRestartParser, LifeRestartRule, PropertyItemNode, RootNode, StatementNode};
use std::{collections::BTreeMap, num::NonZeroUsize, str::FromStr};
use yggdrasil_rt::YggdrasilParser;

pub use self::properties::{PropertyItem, PropertyManager};

mod parser;
mod properties;

#[derive(Clone, Debug)]
pub struct LifeVM {
    property: PropertyManager,
}
