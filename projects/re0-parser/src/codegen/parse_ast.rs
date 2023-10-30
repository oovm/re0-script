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
            top_statement: pair
                .take_tagged_items::<TopStatementNode>(Cow::Borrowed("top_statement"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
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
impl YggdrasilNode for TopStatementNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::Eos(s) => s.get_range(),
            Self::EventGroup(s) => s.get_range(),
            Self::EventStatement(s) => s.get_range(),
            Self::PropertyStatement(s) => s.get_range(),
            Self::TraitGroup(s) => s.get_range(),
            Self::TraitStatement(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<EosNode>(Cow::Borrowed("eos")) {
            return Ok(Self::Eos(s));
        }
        if let Ok(s) = pair.take_tagged_one::<EventGroupNode>(Cow::Borrowed("event_group")) {
            return Ok(Self::EventGroup(s));
        }
        if let Ok(s) = pair.take_tagged_one::<EventStatementNode>(Cow::Borrowed("event_statement")) {
            return Ok(Self::EventStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<PropertyStatementNode>(Cow::Borrowed("property_statement")) {
            return Ok(Self::PropertyStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<TraitGroupNode>(Cow::Borrowed("trait_group")) {
            return Ok(Self::TraitGroup(s));
        }
        if let Ok(s) = pair.take_tagged_one::<TraitStatementNode>(Cow::Borrowed("trait_statement")) {
            return Ok(Self::TraitStatement(s));
        }
        Err(YggdrasilError::invalid_node(LifeRestartRule::TopStatement, _span))
    }
}
#[automatically_derived]
impl FromStr for TopStatementNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::TopStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for EosNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for EosNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::EOS)?)
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
            property_item: pair
                .take_tagged_items::<PropertyItemNode>(Cow::Borrowed("property_item"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
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
impl YggdrasilNode for PropertyItemNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::DescriptionStatement(s) => s.get_range(),
            Self::EnumerateStatement(s) => s.get_range(),
            Self::Eos(s) => s.get_range(),
            Self::IdStatement(s) => s.get_range(),
            Self::OptionStatement(s) => s.get_range(),
            Self::RequirementStatement(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<DescriptionStatementNode>(Cow::Borrowed("description_statement")) {
            return Ok(Self::DescriptionStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<EnumerateStatementNode>(Cow::Borrowed("enumerate_statement")) {
            return Ok(Self::EnumerateStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<EosNode>(Cow::Borrowed("eos")) {
            return Ok(Self::Eos(s));
        }
        if let Ok(s) = pair.take_tagged_one::<IdStatementNode>(Cow::Borrowed("id_statement")) {
            return Ok(Self::IdStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<OptionStatementNode>(Cow::Borrowed("option_statement")) {
            return Ok(Self::OptionStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<RequirementStatementNode>(Cow::Borrowed("requirement_statement")) {
            return Ok(Self::RequirementStatement(s));
        }
        Err(YggdrasilError::invalid_node(LifeRestartRule::PropertyItem, _span))
    }
}
#[automatically_derived]
impl FromStr for PropertyItemNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::PropertyItem)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for EnumerateStatementNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            variant: pair.take_tagged_items::<IdentifierNode>(Cow::Borrowed("variant")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
        })
    }
}
#[automatically_derived]
impl FromStr for EnumerateStatementNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::EnumerateStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for OptionStatementNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            option_item: pair
                .take_tagged_items::<OptionItemNode>(Cow::Borrowed("option_item"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
        })
    }
}
#[automatically_derived]
impl FromStr for OptionStatementNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::OptionStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for OptionItemNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            variant: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("variant"))?,
            weight: pair.take_tagged_option::<IntegerNode>(Cow::Borrowed("weight")),
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
        })
    }
}
#[automatically_derived]
impl FromStr for OptionItemNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::OptionItem)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TraitGroupNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            trait_statement: pair
                .take_tagged_items::<TraitStatementNode>(Cow::Borrowed("trait_statement"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
        })
    }
}
#[automatically_derived]
impl FromStr for TraitGroupNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::TraitGroup)?)
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
            trait_item: pair.take_tagged_items::<TraitItemNode>(Cow::Borrowed("trait_item")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
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
impl YggdrasilNode for TraitItemNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::DescriptionStatement(s) => s.get_range(),
            Self::EffectStatement(s) => s.get_range(),
            Self::IdStatement(s) => s.get_range(),
            Self::RequirementStatement(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<DescriptionStatementNode>(Cow::Borrowed("description_statement")) {
            return Ok(Self::DescriptionStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<EffectStatementNode>(Cow::Borrowed("effect_statement")) {
            return Ok(Self::EffectStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<IdStatementNode>(Cow::Borrowed("id_statement")) {
            return Ok(Self::IdStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<RequirementStatementNode>(Cow::Borrowed("requirement_statement")) {
            return Ok(Self::RequirementStatement(s));
        }
        Err(YggdrasilError::invalid_node(LifeRestartRule::TraitItem, _span))
    }
}
#[automatically_derived]
impl FromStr for TraitItemNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::TraitItem)?)
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
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
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
impl YggdrasilNode for RequirementStatementNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            statement: pair.take_tagged_items::<StatementNode>(Cow::Borrowed("statement")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
        })
    }
}
#[automatically_derived]
impl FromStr for RequirementStatementNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::RequirementStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for EffectStatementNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            statement: pair.take_tagged_items::<StatementNode>(Cow::Borrowed("statement")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
        })
    }
}
#[automatically_derived]
impl FromStr for EffectStatementNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::EffectStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for EventGroupNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            event_statement: pair
                .take_tagged_items::<EventStatementNode>(Cow::Borrowed("event_statement"))
                .collect::<Result<Vec<_>, _>>()?,
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
        })
    }
}
#[automatically_derived]
impl FromStr for EventGroupNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::EventGroup)?)
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
            event_item: pair.take_tagged_items::<EventItemNode>(Cow::Borrowed("event_item")).collect::<Result<Vec<_>, _>>()?,
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
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
impl YggdrasilNode for EventItemNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::DescriptionStatement(s) => s.get_range(),
            Self::IdStatement(s) => s.get_range(),
            Self::OptionStatement(s) => s.get_range(),
            Self::RequirementStatement(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<DescriptionStatementNode>(Cow::Borrowed("description_statement")) {
            return Ok(Self::DescriptionStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<IdStatementNode>(Cow::Borrowed("id_statement")) {
            return Ok(Self::IdStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<OptionStatementNode>(Cow::Borrowed("option_statement")) {
            return Ok(Self::OptionStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<RequirementStatementNode>(Cow::Borrowed("requirement_statement")) {
            return Ok(Self::RequirementStatement(s));
        }
        Err(YggdrasilError::invalid_node(LifeRestartRule::EventItem, _span))
    }
}
#[automatically_derived]
impl FromStr for EventItemNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::EventItem)?)
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
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
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
impl YggdrasilNode for IdStatementNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            integer: pair.take_tagged_one::<IntegerNode>(Cow::Borrowed("integer"))?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
        })
    }
}
#[automatically_derived]
impl FromStr for IdStatementNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::IdStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DescriptionStatementNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            string: pair.take_tagged_items::<StringNode>(Cow::Borrowed("string")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
        })
    }
}
#[automatically_derived]
impl FromStr for DescriptionStatementNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::DescriptionStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StatementNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::Eos(s) => s.get_range(),
            Self::Expression(s) => s.get_range(),
            Self::IfStatement(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<EosNode>(Cow::Borrowed("eos")) {
            return Ok(Self::Eos(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ExpressionNode>(Cow::Borrowed("expression")) {
            return Ok(Self::Expression(s));
        }
        if let Ok(s) = pair.take_tagged_one::<IfStatementNode>(Cow::Borrowed("if_statement")) {
            return Ok(Self::IfStatement(s));
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
impl YggdrasilNode for IfStatementNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            if_else_block: pair.take_tagged_option::<IfElseBlockNode>(Cow::Borrowed("if_else_block")),
            if_then_block: pair.take_tagged_one::<IfThenBlockNode>(Cow::Borrowed("if_then_block"))?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
        })
    }
}
#[automatically_derived]
impl FromStr for IfStatementNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::IfStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for IfThenBlockNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            expression: pair.take_tagged_items::<ExpressionNode>(Cow::Borrowed("expression")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
        })
    }
}
#[automatically_derived]
impl FromStr for IfThenBlockNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::IfThenBlock)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for IfElseBlockNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            expression: pair.take_tagged_items::<ExpressionNode>(Cow::Borrowed("expression")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
        })
    }
}
#[automatically_derived]
impl FromStr for IfElseBlockNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::IfElseBlock)?)
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
            infix: pair.take_tagged_one::<InfixNode>(Cow::Borrowed("infix"))?,
            term: pair.take_tagged_items::<TermNode>(Cow::Borrowed("term")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
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
impl YggdrasilNode for TermNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            atomic: pair.take_tagged_one::<AtomicNode>(Cow::Borrowed("atomic"))?,
            prefix: pair.take_tagged_items::<PrefixNode>(Cow::Borrowed("prefix")).collect::<Result<Vec<_>, _>>()?,
            suffix: pair.take_tagged_items::<SuffixNode>(Cow::Borrowed("suffix")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
        })
    }
}
#[automatically_derived]
impl FromStr for TermNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::Term)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for PrefixNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::Prefix0 => None,
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Some(_) = pair.find_first_tag("prefix_0") {
            return Ok(Self::Prefix0);
        }
        Err(YggdrasilError::invalid_node(LifeRestartRule::Prefix, _span))
    }
}
#[automatically_derived]
impl FromStr for PrefixNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::Prefix)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for InfixNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::Add => None,
            Self::AddAssign => None,
            Self::And => None,
            Self::EQ => None,
            Self::GEQ => None,
            Self::GT => None,
            Self::Has => None,
            Self::LEQ => None,
            Self::LT => None,
            Self::NE => None,
            Self::Or => None,
            Self::Sub => None,
            Self::SubAssign => None,
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Some(_) = pair.find_first_tag("add") {
            return Ok(Self::Add);
        }
        if let Some(_) = pair.find_first_tag("add_assign") {
            return Ok(Self::AddAssign);
        }
        if let Some(_) = pair.find_first_tag("and") {
            return Ok(Self::And);
        }
        if let Some(_) = pair.find_first_tag("eq") {
            return Ok(Self::EQ);
        }
        if let Some(_) = pair.find_first_tag("geq") {
            return Ok(Self::GEQ);
        }
        if let Some(_) = pair.find_first_tag("gt") {
            return Ok(Self::GT);
        }
        if let Some(_) = pair.find_first_tag("has") {
            return Ok(Self::Has);
        }
        if let Some(_) = pair.find_first_tag("leq") {
            return Ok(Self::LEQ);
        }
        if let Some(_) = pair.find_first_tag("lt") {
            return Ok(Self::LT);
        }
        if let Some(_) = pair.find_first_tag("ne") {
            return Ok(Self::NE);
        }
        if let Some(_) = pair.find_first_tag("or") {
            return Ok(Self::Or);
        }
        if let Some(_) = pair.find_first_tag("sub") {
            return Ok(Self::Sub);
        }
        if let Some(_) = pair.find_first_tag("sub_assign") {
            return Ok(Self::SubAssign);
        }
        Err(YggdrasilError::invalid_node(LifeRestartRule::Infix, _span))
    }
}
#[automatically_derived]
impl FromStr for InfixNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::Infix)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for SuffixNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::Suffix0 => None,
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Some(_) = pair.find_first_tag("suffix_0") {
            return Ok(Self::Suffix0);
        }
        Err(YggdrasilError::invalid_node(LifeRestartRule::Suffix, _span))
    }
}
#[automatically_derived]
impl FromStr for SuffixNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::Suffix)?)
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
            Self::List(s) => s.get_range(),
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
        if let Ok(s) = pair.take_tagged_one::<ListNode>(Cow::Borrowed("list")) {
            return Ok(Self::List(s));
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
impl YggdrasilNode for ListNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            comma: pair.take_tagged_items::<CommaNode>(Cow::Borrowed("comma")).collect::<Result<Vec<_>, _>>()?,
            expression: pair.take_tagged_items::<ExpressionNode>(Cow::Borrowed("expression")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
        })
    }
}
#[automatically_derived]
impl FromStr for ListNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::List)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StringNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::StringRaw(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<StringRawNode>(Cow::Borrowed("string_raw")) {
            return Ok(Self::StringRaw(s));
        }
        Err(YggdrasilError::invalid_node(LifeRestartRule::String, _span))
    }
}
#[automatically_derived]
impl FromStr for StringNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::String)?)
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
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
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
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as usize, end: _span.end() as usize } })
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
            string_item: pair
                .take_tagged_items::<StringItemNode>(Cow::Borrowed("string_item"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
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
            span: Range { start: _span.start() as usize, end: _span.end() as usize },
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
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
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
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as usize, end: _span.end() as usize } })
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
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as usize, end: _span.end() as usize } })
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
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as usize, end: _span.end() as usize } })
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
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as usize, end: _span.end() as usize } })
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
            return Ok(Self::False);
        }
        if let Some(_) = pair.find_first_tag("true") {
            return Ok(Self::True);
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
impl YggdrasilNode for CommaNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for CommaNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::COMMA)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ColonNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for ColonNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::COLON)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwPropertyNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for KwPropertyNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::KW_PROPERTY)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwTraitGroupNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for KwTraitGroupNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::KW_TRAIT_GROUP)?)
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
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
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
impl YggdrasilNode for KwEventGroupNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for KwEventGroupNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::KW_EVENT_GROUP)?)
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
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
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
impl YggdrasilNode for KwIdNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for KwIdNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::KW_ID)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwDescriptionNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for KwDescriptionNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::KW_DESCRIPTION)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwRequirementNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for KwRequirementNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::KW_REQUIREMENT)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwEffectNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for KwEffectNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::KW_EFFECT)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwEnumerateNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for KwEnumerateNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::KW_ENUMERATE)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwOptionsNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for KwOptionsNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::KW_OPTIONS)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwIfNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for KwIfNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::KW_IF)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwElseIfNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for KwElseIfNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::KW_ELSE_IF)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwElseNode {
    type Rule = LifeRestartRule;

    fn get_range(&self) -> Option<Range<usize>> {
        Some(Range { start: self.span.start as usize, end: self.span.end as usize })
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for KwElseNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::KW_ELSE)?)
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
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
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
        Ok(Self { span: Range { start: _span.start() as usize, end: _span.end() as usize } })
    }
}
#[automatically_derived]
impl FromStr for CommentNode {
    type Err = YggdrasilError<LifeRestartRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<LifeRestartRule>> {
        Self::from_cst(LifeRestartParser::parse_cst(input, LifeRestartRule::Comment)?)
    }
}
