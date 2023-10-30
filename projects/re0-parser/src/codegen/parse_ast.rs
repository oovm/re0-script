use super::*;
#[automatically_derived]
impl YggdrasilNode for RootNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            statement: pair.take_tagged_items::<StatementNode>(Cow::Borrowed("statement")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for RootNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::Root)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StatementNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::EventStatement(s) => s.get_range(),
            Self::PropertyStatement(s) => s.get_range(),
            Self::TraitStatement(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<EventStatementNode>(Cow::Borrowed("event_statement")) {
            return Ok(Self::EventStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<PropertyStatementNode>(Cow::Borrowed("property_statement")) {
            return Ok(Self::PropertyStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<TraitStatementNode>(Cow::Borrowed("trait_statement")) {
            return Ok(Self::TraitStatement(s));
        }
        Err(YggdrasilError::invalid_node(LifeRestartRule::Statement, _span))
    }
}
#[automatically_derived]
impl FromStr for StatementNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::Statement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for PropertyStatementNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            property_block: pair.take_tagged_one::<PropertyBlockNode>(Cow::Borrowed("property_block"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for PropertyStatementNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::PropertyStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for PropertyBlockNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for PropertyBlockNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::PropertyBlock)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TraitStatementNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            trait_block: pair.take_tagged_one::<TraitBlockNode>(Cow::Borrowed("trait_block"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TraitStatementNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::TraitStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TraitBlockNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            trait_property: pair.take_tagged_items::<TraitPropertyNode>(Cow::Borrowed("trait_property")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TraitBlockNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::TraitBlock)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TraitPropertyNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            atomic: pair.take_tagged_one::<AtomicNode>(Cow::Borrowed("atomic"))?,
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TraitPropertyNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::TraitProperty)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for EventStatementNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            event_block: pair.take_tagged_one::<EventBlockNode>(Cow::Borrowed("event_block"))?,
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for EventStatementNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::EventStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for EventBlockNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            event_property: pair.take_tagged_items::<EventPropertyNode>(Cow::Borrowed("event_property")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for EventBlockNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::EventBlock)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for EventPropertyNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            event_block: pair.take_tagged_option::<EventBlockNode>(Cow::Borrowed("event_block")),
            expression: pair.take_tagged_one::<ExpressionNode>(Cow::Borrowed("expression"))?,
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for EventPropertyNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::EventProperty)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ExpressionNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            atomic: pair.take_tagged_one::<AtomicNode>(Cow::Borrowed("atomic"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ExpressionNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::Expression)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for AtomicNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::Boolean(s) => s.get_range(),
            Self::Identifier(s) => s.get_range(),
            Self::Integer(s) => s.get_range(),
            Self::Range(s) => s.get_range(),
            Self::RangeExact(s) => s.get_range(),
            Self::StringNormal(s) => s.get_range(),
            Self::StringRaw(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<BooleanNode>(Cow::Borrowed("boolean")) {
            return Ok(Self::Boolean(s));
        }
        if let Ok(s) = pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier")) {
            return Ok(Self::Identifier(s));
        }
        if let Ok(s) = pair.take_tagged_one::<IntegerNode>(Cow::Borrowed("integer")) {
            return Ok(Self::Integer(s));
        }
        if let Ok(s) = pair.take_tagged_one::<RangeNode>(Cow::Borrowed("range")) {
            return Ok(Self::Range(s));
        }
        if let Ok(s) = pair.take_tagged_one::<RangeExactNode>(Cow::Borrowed("range_exact")) {
            return Ok(Self::RangeExact(s));
        }
        if let Ok(s) = pair.take_tagged_one::<StringNormalNode>(Cow::Borrowed("string_normal")) {
            return Ok(Self::StringNormal(s));
        }
        if let Ok(s) = pair.take_tagged_one::<StringRawNode>(Cow::Borrowed("string_raw")) {
            return Ok(Self::StringRaw(s));
        }
        Err(YggdrasilError::invalid_node(LifeRestartRule::Atomic, _span))
    }
}
#[automatically_derived]
impl FromStr for AtomicNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::Atomic)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for NegationExpressionNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            expression: pair.take_tagged_one::<ExpressionNode>(Cow::Borrowed("expression"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for NegationExpressionNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::NegationExpression)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ComparisonExpressionNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            comparison_operator: pair.take_tagged_one::<ComparisonOperatorNode>(Cow::Borrowed("comparison_operator"))?,
            expression: pair.take_tagged_one::<ExpressionNode>(Cow::Borrowed("expression"))?,
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ComparisonExpressionNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::ComparisonExpression)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for LogicalExpressionNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            expression: pair.take_tagged_items::<ExpressionNode>(Cow::Borrowed("expression")).collect::<Result<Vec<_>, _>>()?,
            logical_operator: pair.take_tagged_one::<LogicalOperatorNode>(Cow::Borrowed("logical_operator"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for LogicalExpressionNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::LogicalExpression)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ComparisonOperatorNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::ComparisonOperator0 => None,
            Self::ComparisonOperator1 => None,
            Self::ComparisonOperator2 => None,
            Self::ComparisonOperator3 => None,
            Self::ComparisonOperator4 => None,
            Self::ComparisonOperator5 => None,
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Some(_) = pair.find_first_tag("comparison_operator_0") {
            return Ok(Self::ComparisonOperator0)
        }
        if let Some(_) = pair.find_first_tag("comparison_operator_1") {
            return Ok(Self::ComparisonOperator1)
        }
        if let Some(_) = pair.find_first_tag("comparison_operator_2") {
            return Ok(Self::ComparisonOperator2)
        }
        if let Some(_) = pair.find_first_tag("comparison_operator_3") {
            return Ok(Self::ComparisonOperator3)
        }
        if let Some(_) = pair.find_first_tag("comparison_operator_4") {
            return Ok(Self::ComparisonOperator4)
        }
        if let Some(_) = pair.find_first_tag("comparison_operator_5") {
            return Ok(Self::ComparisonOperator5)
        }
        Err(YggdrasilError::invalid_node(LifeRestartRule::ComparisonOperator, _span))
    }
}
#[automatically_derived]
impl FromStr for ComparisonOperatorNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::ComparisonOperator)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for LogicalOperatorNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::LogicalOperator0 => None,
            Self::LogicalOperator1 => None,
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Some(_) = pair.find_first_tag("logical_operator_0") {
            return Ok(Self::LogicalOperator0)
        }
        if let Some(_) = pair.find_first_tag("logical_operator_1") {
            return Ok(Self::LogicalOperator1)
        }
        Err(YggdrasilError::invalid_node(LifeRestartRule::LogicalOperator, _span))
    }
}
#[automatically_derived]
impl FromStr for LogicalOperatorNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::LogicalOperator)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StringRawNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            string_raw_text: pair.take_tagged_one::<StringRawTextNode>(Cow::Borrowed("string_raw_text"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for StringRawNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::StringRaw)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StringRawTextNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            text: pair.get_string(),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for StringRawTextNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::StringRawText)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StringNormalNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            string_item: pair.take_tagged_items::<StringItemNode>(Cow::Borrowed("string_item")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for StringNormalNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::StringNormal)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StringItemNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::EscapedCharacter(s) => s.get_range(),
            Self::EscapedUnicode(s) => s.get_range(),
            Self::TextAny(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<EscapedCharacterNode>(Cow::Borrowed("escaped_character")) {
            return Ok(Self::EscapedCharacter(s));
        }
        if let Ok(s) = pair.take_tagged_one::<EscapedUnicodeNode>(Cow::Borrowed("escaped_unicode")) {
            return Ok(Self::EscapedUnicode(s));
        }
        if let Ok(s) = pair.take_tagged_one::<TextAnyNode>(Cow::Borrowed("text_any")) {
            return Ok(Self::TextAny(s));
        }
        Err(YggdrasilError::invalid_node(LifeRestartRule::StringItem, _span))
    }
}
#[automatically_derived]
impl FromStr for StringItemNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::StringItem)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for EscapedUnicodeNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            hex: pair.take_tagged_one::<HexNode>(Cow::Borrowed("hex"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for EscapedUnicodeNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::EscapedUnicode)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for EscapedCharacterNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for EscapedCharacterNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::EscapedCharacter)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for HexNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            text: pair.get_string(),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for HexNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::HEX)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TextAnyNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            text: pair.get_string(),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TextAnyNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::TextAny)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for IdentifierNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            text: pair.get_string(),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for IdentifierNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::Identifier)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for IntegerNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            text: pair.get_string(),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for IntegerNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::Integer)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for RangeExactNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            integer: pair.take_tagged_one::<IntegerNode>(Cow::Borrowed("integer"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for RangeExactNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::RangeExact)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for RangeNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            max: pair.take_tagged_option::<IntegerNode>(Cow::Borrowed("max")),
            min: pair.take_tagged_option::<IntegerNode>(Cow::Borrowed("min")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for RangeNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::Range)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for BooleanNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::False => None,
            Self::True => None,
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Some(_) = pair.find_first_tag("false") {
            return Ok(Self::False)
        }
        if let Some(_) = pair.find_first_tag("true") {
            return Ok(Self::True)
        }
        Err(YggdrasilError::invalid_node(LifeRestartRule::Boolean, _span))
    }
}
#[automatically_derived]
impl FromStr for BooleanNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::Boolean)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwAttributeNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for KwAttributeNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::KW_ATTRIBUTE)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwTraitNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for KwTraitNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::KW_TRAIT)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwEventNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for KwEventNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::KW_EVENT)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for WhiteSpaceNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for WhiteSpaceNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::WhiteSpace)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for CommentNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for CommentNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::Comment)?)
    }
}
