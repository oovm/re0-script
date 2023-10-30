#![allow(dead_code, unused_imports, non_camel_case_types)]
#![allow(missing_docs, rustdoc::missing_crate_level_docs)]
#![allow(clippy::unnecessary_cast)]
#![doc = include_str!("readme.md")]

mod parse_cst;
mod parse_ast;

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
    PropertyStatement,
    PropertyBlock,
    TraitStatement,
    TraitBlock,
    TraitProperty,
    EventStatement,
    EventBlock,
    EventProperty,
    Expression,
    Atomic,
    NegationExpression,
    ComparisonExpression,
    LogicalExpression,
    ComparisonOperator,
    LogicalOperator,
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
    KW_ATTRIBUTE,
    KW_TRAIT,
    KW_EVENT,
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
            Self::PropertyStatement => "",
            Self::PropertyBlock => "",
            Self::TraitStatement => "",
            Self::TraitBlock => "",
            Self::TraitProperty => "",
            Self::EventStatement => "",
            Self::EventBlock => "",
            Self::EventProperty => "",
            Self::Expression => "",
            Self::Atomic => "",
            Self::NegationExpression => "",
            Self::ComparisonExpression => "",
            Self::LogicalExpression => "",
            Self::ComparisonOperator => "",
            Self::LogicalOperator => "",
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
            Self::KW_ATTRIBUTE => "",
            Self::KW_TRAIT => "",
            Self::KW_EVENT => "",
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
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StatementNode {
    EventStatement(EventStatementNode),
    PropertyStatement(PropertyStatementNode),
    TraitStatement(TraitStatementNode),
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PropertyStatementNode {
    pub identifier: IdentifierNode,
    pub property_block: PropertyBlockNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PropertyBlockNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraitStatementNode {
    pub identifier: IdentifierNode,
    pub trait_block: TraitBlockNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraitBlockNode {
    pub trait_property: Vec<TraitPropertyNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraitPropertyNode {
    pub atomic: AtomicNode,
    pub identifier: IdentifierNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EventStatementNode {
    pub event_block: EventBlockNode,
    pub identifier: IdentifierNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EventBlockNode {
    pub event_property: Vec<EventPropertyNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EventPropertyNode {
    pub event_block: Option<EventBlockNode>,
    pub expression: ExpressionNode,
    pub identifier: IdentifierNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressionNode {
    pub atomic: AtomicNode,
    pub span: Range<u32>,
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
pub struct NegationExpressionNode {
    pub expression: ExpressionNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComparisonExpressionNode {
    pub comparison_operator: ComparisonOperatorNode,
    pub expression: ExpressionNode,
    pub identifier: IdentifierNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LogicalExpressionNode {
    pub expression: Vec<ExpressionNode>,
    pub logical_operator: LogicalOperatorNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ComparisonOperatorNode {
    ComparisonOperator0,
    ComparisonOperator1,
    ComparisonOperator2,
    ComparisonOperator3,
    ComparisonOperator4,
    ComparisonOperator5,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LogicalOperatorNode {
    LogicalOperator0,
    LogicalOperator1,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringRawNode {
    pub string_raw_text: StringRawTextNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringRawTextNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringNormalNode {
    pub string_item: Vec<StringItemNode>,
    pub span: Range<u32>,
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
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EscapedCharacterNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HexNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextAnyNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentifierNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IntegerNode {
    pub text: String,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeExactNode {
    pub integer: IntegerNode,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeNode {
    pub max: Option<IntegerNode>,
    pub min: Option<IntegerNode>,
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BooleanNode {
    False,
    True,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwAttributeNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwTraitNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KwEventNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhiteSpaceNode {
    pub span: Range<u32>,
}
#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommentNode {
    pub span: Range<u32>,
}
