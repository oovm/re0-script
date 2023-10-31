use crate::codegen::{PropertyItemNode, RootNode};
use std::{collections::BTreeMap, num::NonZeroUsize, ops::Range, str::FromStr};

use crate::errors::LifeErrorKind;
use url::Url;

pub use self::{
    characters::{CharacterItem, CharacterManager},
    identifier::Identifier,
    properties::{PropertyItem, PropertyManager},
    stories::{StoryItem, StoryManager},
    talents::{TalentItem, TalentManager},
};
use crate::LifeError;
mod characters;
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
