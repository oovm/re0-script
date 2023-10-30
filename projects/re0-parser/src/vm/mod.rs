use crate::codegen::{PropertyItemNode, RootNode, StatementNode};
use std::{collections::BTreeMap, num::NonZeroUsize, ops::Range, str::FromStr};

use crate::errors::LifeErrorKind;
use url::Url;

pub use self::{
    identifier::Identifier,
    properties::{PropertyItem, PropertyManager},
    stories::{StoryItem, StoryManager},
    talents::{TalentItem, TalentManager},
};
use crate::LifeError;
mod identifier;
mod parser;
mod properties;
mod stories;
mod talents;

/// All data of life restart game
#[derive(Clone, Debug, Default)]
pub struct LifeVM {
    property: PropertyManager,
    talent: TalentManager,
    story: StoryManager,
}
