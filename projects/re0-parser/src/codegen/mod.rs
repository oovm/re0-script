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
    TopStatement,
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
    Statement,
    IfStatement,
    IfThenBlock,
    IfElseIfBlock,
    IfElseBlock,
    Expression,
    Term,
    Prefix,
    Infix,
    Suffix,
    Atomic,
    List,
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
    KW_IF,
    KW_ELSE_IF,
    KW_ELSE,
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
            Self::TopStatement => "",
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
            Self::Statement => "",
            Self::IfStatement => "",
            Self::IfThenBlock => "",
            Self::IfElseIfBlock => "",
            Self::IfElseBlock => "",
            Self::Expression => "",
            Self::Term => "",
            Self::Prefix => "",
            Self::Infix => "",
            Self::Suffix => "",
            Self::Atomic => "",
            Self::List => "",
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
            Self::KW_IF => "",
            Self::KW_ELSE_IF => "",
            Self::KW_ELSE => "",
            Self::WhiteSpace => "",
            Self::Comment => "",
            _ => "",
        }
    }
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RootNode {
    pub top_statement: Vec<TopStatementNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TopStatementNode {
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
    pub statement: Vec<StatementNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EffectStatementNode {
    pub statement: Vec<StatementNode>,
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
pub enum StatementNode {
    Eos(EosNode),
    Expression(ExpressionNode),
    IfStatement(IfStatementNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IfStatementNode {
    pub if_else_block: Option<IfElseBlockNode>,
    pub if_else_if_block: Option<IfElseIfBlockNode>,
    pub if_then_block: IfThenBlockNode,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IfThenBlockNode {
    pub expression: ExpressionNode,
    pub statement: Vec<StatementNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IfElseIfBlockNode {
    pub expression: ExpressionNode,
    pub statement: Vec<StatementNode>,
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IfElseBlockNode {
    pub statement: Vec<StatementNode>,
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
pub enum PrefixNode {
    Prefix0,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InfixNode {
    Add,
    AddAssign,
    And,
    EQ,
    GEQ,
    GT,
    Has,
    LEQ,
    LT,
    NE,
    Or,
    Sub,
    SubAssign,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SuffixNode {
    Suffix0,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AtomicNode {
    Boolean(BooleanNode),
    Identifier(IdentifierNode),
    Integer(IntegerNode),
    List(ListNode),
    StringNormal(StringNormalNode),
    StringRaw(StringRawNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ListNode {
    pub atomic: Vec<AtomicNode>,
    pub comma: Vec<CommaNode>,
    pub span: Range<usize>,
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
pub struct KwIfNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwElseIfNode {
    pub span: Range<usize>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwElseNode {
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
