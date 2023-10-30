#![allow(dead_code, unused_imports, non_camel_case_types)]
#![allow(missing_docs, rustdoc::missing_crate_level_docs)]
#![allow(clippy::unnecessary_cast)]
#![doc = include_str!("readme.md")]

mod parse_ast;
mod parse_cst;

use core::str::FromStr;
use std::{borrow::Cow, ops::Range, sync::OnceLock};
use yggdrasil_rt::*;

type Input<'i> = Box<State<'i, LifeRestartRule>>;
type Output<'i> = Result<Box<State<'i, LifeRestartRule>>, Box<State<'i, LifeRestartRule>>>;

#[doc = include_str!("railway.min.svg")]
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LifeRestartParser {}

impl YggdrasilParser for LifeRestartParser {
    type Rule = LifeRestartRule;
    fn parse_cst(input: &str, rule: Self::Rule) -> OutputResult<LifeRestartRule> {
        self::parse_cst::parse_cst(input, rule)
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LifeRestartRule {
    Root,
    Statement,
    EOS,
    PropertyStatement,
    PropertyItem,
    EnumerateStatement,
    OptionStatement,
    OptionItem,
    TraitGroup,
    TraitStatement,
    TraitItem,
    TraitProperty,
    RequirementStatement,
    EffectStatement,
    EventGroup,
    EventStatement,
    EventItem,
    EventProperty,
    IdStatement,
    DescriptionStatement,
    Expression,
    Term,
    Atomic,
    Prefix,
    Infix,
    Suffix,
    String,
    StringRaw,
    StringRawText,
    StringNormal,
    StringItem,
    EscapedUnicode,
    EscapedCharacter,
    HEX,
    TextAny,
    Identifier,
    Integer,
    RangeExact,
    Range,
    Boolean,
    COMMA,
    COLON,
    KW_PROPERTY,
    KW_TRAIT_GROUP,
    KW_TRAIT,
    KW_EVENT_GROUP,
    KW_EVENT,
    KW_ID,
    KW_DESCRIPTION,
    KW_REQUIREMENT,
    KW_EFFECT,
    KW_ENUMERATE,
    KW_OPTIONS,
    WhiteSpace,
    Comment,
    /// Label for unnamed text literal
    HiddenText,
}

impl YggdrasilRule for LifeRestartRule {
    fn is_ignore(&self) -> bool {
        matches!(self, Self::HiddenText | Self::WhiteSpace | Self::Comment)
    }

    fn get_style(&self) -> &'static str {
        match self {
            Self::Root => "",
            Self::Statement => "",
            Self::EOS => "",
            Self::PropertyStatement => "",
            Self::PropertyItem => "",
            Self::EnumerateStatement => "",
            Self::OptionStatement => "",
            Self::OptionItem => "",
            Self::TraitGroup => "",
            Self::TraitStatement => "",
            Self::TraitItem => "",
            Self::TraitProperty => "",
            Self::RequirementStatement => "",
            Self::EffectStatement => "",
            Self::EventGroup => "",
            Self::EventStatement => "",
            Self::EventItem => "",
            Self::EventProperty => "",
            Self::IdStatement => "",
            Self::DescriptionStatement => "",
            Self::Expression => "",
            Self::Term => "",
            Self::Atomic => "",
            Self::Prefix => "",
            Self::Infix => "",
            Self::Suffix => "",
            Self::String => "",
            Self::StringRaw => "",
            Self::StringRawText => "",
            Self::StringNormal => "",
            Self::StringItem => "",
            Self::EscapedUnicode => "",
            Self::EscapedCharacter => "",
            Self::HEX => "",
            Self::TextAny => "",
            Self::Identifier => "",
            Self::Integer => "",
            Self::RangeExact => "",
            Self::Range => "",
            Self::Boolean => "",
            Self::COMMA => "",
            Self::COLON => "",
            Self::KW_PROPERTY => "",
            Self::KW_TRAIT_GROUP => "",
            Self::KW_TRAIT => "",
            Self::KW_EVENT_GROUP => "",
            Self::KW_EVENT => "",
            Self::KW_ID => "",
            Self::KW_DESCRIPTION => "",
            Self::KW_REQUIREMENT => "",
            Self::KW_EFFECT => "",
            Self::KW_ENUMERATE => "",
            Self::KW_OPTIONS => "",
            Self::WhiteSpace => "",
            Self::Comment => "",
            _ => "",
        }
    }
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RootNode {
    pub statement: Vec<StatementNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StatementNode {
    Eos(EosNode),
    EventGroup(EventGroupNode),
    EventStatement(EventStatementNode),
    PropertyStatement(PropertyStatementNode),
    TraitGroup(TraitGroupNode),
    TraitStatement(TraitStatementNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EosNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PropertyStatementNode {
    pub identifier: IdentifierNode,
    pub property_item: Vec<PropertyItemNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PropertyItemNode {
    DescriptionStatement(DescriptionStatementNode),
    EnumerateStatement(EnumerateStatementNode),
    Eos(EosNode),
    IdStatement(IdStatementNode),
    OptionStatement(OptionStatementNode),
    RequirementStatement(RequirementStatementNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EnumerateStatementNode {
    pub variant: Vec<IdentifierNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OptionStatementNode {
    pub option_item: Vec<OptionItemNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OptionItemNode {
    pub variant: IdentifierNode,
    pub weight: Option<IntegerNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraitGroupNode {
    pub identifier: IdentifierNode,
    pub trait_statement: Vec<TraitStatementNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraitStatementNode {
    pub identifier: IdentifierNode,
    pub trait_item: Vec<TraitItemNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TraitItemNode {
    DescriptionStatement(DescriptionStatementNode),
    EffectStatement(EffectStatementNode),
    IdStatement(IdStatementNode),
    RequirementStatement(RequirementStatementNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraitPropertyNode {
    pub atomic: AtomicNode,
    pub identifier: IdentifierNode,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RequirementStatementNode {
    pub expression: Vec<ExpressionNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EffectStatementNode {
    pub expression: Vec<ExpressionNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EventGroupNode {
    pub event_statement: Vec<EventStatementNode>,
    pub identifier: IdentifierNode,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EventStatementNode {
    pub event_item: Vec<EventItemNode>,
    pub identifier: IdentifierNode,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EventItemNode {
    DescriptionStatement(DescriptionStatementNode),
    IdStatement(IdStatementNode),
    OptionStatement(OptionStatementNode),
    RequirementStatement(RequirementStatementNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EventPropertyNode {
    pub identifier: IdentifierNode,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdStatementNode {
    pub integer: IntegerNode,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DescriptionStatementNode {
    pub string: Vec<StringNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressionNode {
    pub infix: InfixNode,
    pub term: Vec<TermNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TermNode {
    pub atomic: AtomicNode,
    pub prefix: Vec<PrefixNode>,
    pub suffix: Vec<SuffixNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AtomicNode {
    Boolean(BooleanNode),
    Identifier(IdentifierNode),
    Integer(IntegerNode),
    Range(RangeNode),
    RangeExact(RangeExactNode),
    StringNormal(StringNormalNode),
    StringRaw(StringRawNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PrefixNode {
    Prefix0,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InfixNode {
    And,
    EQ,
    GEQ,
    GT,
    LEQ,
    LT,
    NE,
    Or,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SuffixNode {
    Suffix0,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StringNode {
    StringRaw(StringRawNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringRawNode {
    pub string_raw_text: StringRawTextNode,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringRawTextNode {
    pub text: String,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringNormalNode {
    pub string_item: Vec<StringItemNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StringItemNode {
    EscapedCharacter(EscapedCharacterNode),
    EscapedUnicode(EscapedUnicodeNode),
    TextAny(TextAnyNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EscapedUnicodeNode {
    pub hex: HexNode,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EscapedCharacterNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HexNode {
    pub text: String,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextAnyNode {
    pub text: String,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentifierNode {
    pub text: String,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IntegerNode {
    pub text: String,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeExactNode {
    pub integer: IntegerNode,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeNode {
    pub max: Option<IntegerNode>,
    pub min: Option<IntegerNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BooleanNode {
    False,
    True,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommaNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColonNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwPropertyNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwTraitGroupNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwTraitNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwEventGroupNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwEventNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwIdNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwDescriptionNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwRequirementNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwEffectNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwEnumerateNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwOptionsNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhiteSpaceNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommentNode {
    pub span: Range<usize>,
}
