use super::*;

pub(super) fn parse_cst(input: &str, rule: LifeRestartRule) -> OutputResult<LifeRestartRule> {
    state(input, |state| match rule {
        LifeRestartRule::Root => parse_root(state),
        LifeRestartRule::Statement => parse_statement(state),
        LifeRestartRule::PropertyStatement => parse_property_statement(state),
        LifeRestartRule::PropertyBlock => parse_property_block(state),
        LifeRestartRule::TraitStatement => parse_trait_statement(state),
        LifeRestartRule::TraitBlock => parse_trait_block(state),
        LifeRestartRule::TraitProperty => parse_trait_property(state),
        LifeRestartRule::EventStatement => parse_event_statement(state),
        LifeRestartRule::EventBlock => parse_event_block(state),
        LifeRestartRule::EventProperty => parse_event_property(state),
        LifeRestartRule::Expression => parse_expression(state),
        LifeRestartRule::Atomic => parse_atomic(state),
        LifeRestartRule::NegationExpression => parse_negation_expression(state),
        LifeRestartRule::ComparisonExpression => parse_comparison_expression(state),
        LifeRestartRule::LogicalExpression => parse_logical_expression(state),
        LifeRestartRule::ComparisonOperator => parse_comparison_operator(state),
        LifeRestartRule::LogicalOperator => parse_logical_operator(state),
        LifeRestartRule::StringRaw => parse_string_raw(state),
        LifeRestartRule::StringRawText => parse_string_raw_text(state),
        LifeRestartRule::StringNormal => parse_string_normal(state),
        LifeRestartRule::StringItem => parse_string_item(state),
        LifeRestartRule::EscapedUnicode => parse_escaped_unicode(state),
        LifeRestartRule::EscapedCharacter => parse_escaped_character(state),
        LifeRestartRule::HEX => parse_hex(state),
        LifeRestartRule::TextAny => parse_text_any(state),
        LifeRestartRule::Identifier => parse_identifier(state),
        LifeRestartRule::Integer => parse_integer(state),
        LifeRestartRule::RangeExact => parse_range_exact(state),
        LifeRestartRule::Range => parse_range(state),
        LifeRestartRule::Boolean => parse_boolean(state),
        LifeRestartRule::KW_ATTRIBUTE => parse_kw_attribute(state),
        LifeRestartRule::KW_TRAIT => parse_kw_trait(state),
        LifeRestartRule::KW_EVENT => parse_kw_event(state),
        LifeRestartRule::WhiteSpace => parse_white_space(state),
        LifeRestartRule::Comment => parse_comment(state),
        LifeRestartRule::HiddenText => unreachable!(),
    })
}

#[inline]
fn parse_root(state: Input) -> Output {
    state.rule(LifeRestartRule::Root, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_statement(s).and_then(|s| s.tag_node("statement")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.end_of_input())
        })
    })
}

#[inline]
fn parse_statement(state: Input) -> Output {
    state.rule(LifeRestartRule::Statement, |s| {
        Err(s)
            .or_else(|s| parse_property_statement(s).and_then(|s| s.tag_node("property_statement")))
            .or_else(|s| parse_trait_statement(s).and_then(|s| s.tag_node("trait_statement")))
            .or_else(|s| parse_event_statement(s).and_then(|s| s.tag_node("event_statement")))
    })
}

#[inline]
fn parse_property_statement(state: Input) -> Output {
    state.rule(LifeRestartRule::PropertyStatement, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_attribute(s))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_property_block(s).and_then(|s| s.tag_node("property_block")))
        })
    })
}

#[inline]
fn parse_property_block(state: Input) -> Output {
    state.rule(LifeRestartRule::PropertyBlock, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}

#[inline]
fn parse_trait_statement(state: Input) -> Output {
    state.rule(LifeRestartRule::TraitStatement, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_trait(s))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_trait_block(s).and_then(|s| s.tag_node("trait_block")))
        })
    })
}

#[inline]
fn parse_trait_block(state: Input) -> Output {
    state.rule(LifeRestartRule::TraitBlock, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_trait_property(s).and_then(|s| s.tag_node("trait_property")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}

#[inline]
fn parse_trait_property(state: Input) -> Output {
    state.rule(LifeRestartRule::TraitProperty, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, ":", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_atomic(s).and_then(|s| s.tag_node("atomic")))
        })
    })
}

#[inline]
fn parse_event_statement(state: Input) -> Output {
    state.rule(LifeRestartRule::EventStatement, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_event(s))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_event_block(s).and_then(|s| s.tag_node("event_block")))
        })
    })
}

#[inline]
fn parse_event_block(state: Input) -> Output {
    state.rule(LifeRestartRule::EventBlock, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_event_property(s).and_then(|s| s.tag_node("event_property")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}

#[inline]
fn parse_event_property(state: Input) -> Output {
    state.rule(LifeRestartRule::EventProperty, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| Err(s).or_else(|s| builtin_text(s, ":", false)).or_else(|s| builtin_text(s, "[", false)))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_expression(s).and_then(|s| s.tag_node("expression")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.optional(|s| {
                        Err(s)
                            .or_else(|s| builtin_text(s, "]", false))
                            .or_else(|s| {
                                s.sequence(|s| {
                                    Ok(s)
                                        .and_then(|s| builtin_text(s, "{", false))
                                        .and_then(|s| builtin_ignore(s))
                                        .and_then(|s| parse_event_block(s).and_then(|s| s.tag_node("event_block")))
                                        .and_then(|s| builtin_ignore(s))
                                        .and_then(|s| builtin_text(s, "}", false))
                                })
                            })
                            .or_else(|s| parse_event_block(s).and_then(|s| s.tag_node("event_block")))
                    })
                })
        })
    })
}

#[inline]
fn parse_expression(state: Input) -> Output {
    state.rule(LifeRestartRule::Expression, |s| parse_atomic(s).and_then(|s| s.tag_node("atomic")))
}

#[inline]
fn parse_atomic(state: Input) -> Output {
    state.rule(LifeRestartRule::Atomic, |s| {
        Err(s)
            .or_else(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
            .or_else(|s| parse_integer(s).and_then(|s| s.tag_node("integer")))
            .or_else(|s| parse_string_raw(s).and_then(|s| s.tag_node("string_raw")))
            .or_else(|s| parse_string_normal(s).and_then(|s| s.tag_node("string_normal")))
            .or_else(|s| parse_boolean(s).and_then(|s| s.tag_node("boolean")))
            .or_else(|s| parse_range_exact(s).and_then(|s| s.tag_node("range_exact")))
            .or_else(|s| parse_range(s).and_then(|s| s.tag_node("range")))
    })
}

#[inline]
fn parse_negation_expression(state: Input) -> Output {
    state.rule(LifeRestartRule::NegationExpression, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "!", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_expression(s).and_then(|s| s.tag_node("expression")))
        })
    })
}

#[inline]
fn parse_comparison_expression(state: Input) -> Output {
    state.rule(LifeRestartRule::ComparisonExpression, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_comparison_operator(s).and_then(|s| s.tag_node("comparison_operator")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_expression(s).and_then(|s| s.tag_node("expression")))
        })
    })
}

#[inline]
fn parse_logical_expression(state: Input) -> Output {
    state.rule(LifeRestartRule::LogicalExpression, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_expression(s).and_then(|s| s.tag_node("expression")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_logical_operator(s).and_then(|s| s.tag_node("logical_operator")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_expression(s).and_then(|s| s.tag_node("expression")))
        })
    })
}

#[inline]
fn parse_comparison_operator(state: Input) -> Output {
    state.rule(LifeRestartRule::ComparisonOperator, |s| {
        Err(s)
            .or_else(|s| builtin_text(s, ">", false).and_then(|s| s.tag_node("comparison_operator_0")))
            .or_else(|s| builtin_text(s, ">=", false).and_then(|s| s.tag_node("comparison_operator_1")))
            .or_else(|s| builtin_text(s, "<", false).and_then(|s| s.tag_node("comparison_operator_2")))
            .or_else(|s| builtin_text(s, "<=", false).and_then(|s| s.tag_node("comparison_operator_3")))
            .or_else(|s| builtin_text(s, "==", false).and_then(|s| s.tag_node("comparison_operator_4")))
            .or_else(|s| builtin_text(s, "!=", false).and_then(|s| s.tag_node("comparison_operator_5")))
    })
}

#[inline]
fn parse_logical_operator(state: Input) -> Output {
    state.rule(LifeRestartRule::LogicalOperator, |s| {
        Err(s)
            .or_else(|s| builtin_text(s, "&&", false).and_then(|s| s.tag_node("logical_operator_0")))
            .or_else(|s| builtin_text(s, "||", false).and_then(|s| s.tag_node("logical_operator_1")))
    })
}

#[inline]
fn parse_string_raw(state: Input) -> Output {
    state.rule(LifeRestartRule::StringRaw, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "'", false))
                .and_then(|s| parse_string_raw_text(s).and_then(|s| s.tag_node("string_raw_text")))
                .and_then(|s| builtin_text(s, "'", false))
        })
    })
}

#[inline]
fn parse_string_raw_text(state: Input) -> Output {
    state.rule(LifeRestartRule::StringRawText, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^([^']*)").unwrap())
        })
    })
}

#[inline]
fn parse_string_normal(state: Input) -> Output {
    state.rule(LifeRestartRule::StringNormal, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "\"", false))
                .and_then(|s| s.repeat(0..4294967295, |s| parse_string_item(s).and_then(|s| s.tag_node("string_item"))))
                .and_then(|s| builtin_text(s, "\"", false))
        })
    })
}

#[inline]
fn parse_string_item(state: Input) -> Output {
    state.rule(LifeRestartRule::StringItem, |s| {
        Err(s)
            .or_else(|s| parse_escaped_unicode(s).and_then(|s| s.tag_node("escaped_unicode")))
            .or_else(|s| parse_escaped_character(s).and_then(|s| s.tag_node("escaped_character")))
            .or_else(|s| parse_text_any(s).and_then(|s| s.tag_node("text_any")))
    })
}

#[inline]
fn parse_escaped_unicode(state: Input) -> Output {
    state.rule(LifeRestartRule::EscapedUnicode, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "\\u{", false))
                .and_then(|s| parse_hex(s).and_then(|s| s.tag_node("hex")))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}

#[inline]
fn parse_escaped_character(state: Input) -> Output {
    state.rule(LifeRestartRule::EscapedCharacter, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(\\\\.)").unwrap())
        })
    })
}

#[inline]
fn parse_hex(state: Input) -> Output {
    state.rule(LifeRestartRule::HEX, |s| {
        s.repeat(1..6, |s| {
            s.sequence(|s| {
                Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                    builtin_regex(s, {
                        static REGEX: OnceLock<Regex> = OnceLock::new();
                        REGEX.get_or_init(|| Regex::new("^([0-9a-fA-F])").unwrap())
                    })
                })
            })
        })
    })
}

#[inline]
fn parse_text_any(state: Input) -> Output {
    state.rule(LifeRestartRule::TextAny, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^([^\"\\\\]+)").unwrap())
        })
    })
}

#[inline]
fn parse_identifier(state: Input) -> Output {
    state.rule(LifeRestartRule::Identifier, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^([_\\p{XID_start}]\\p{XID_continue}*)").unwrap())
        })
    })
}

#[inline]
fn parse_integer(state: Input) -> Output {
    state.rule(LifeRestartRule::Integer, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(0|[1-9][0-9]*)").unwrap())
        })
    })
}

#[inline]
fn parse_range_exact(state: Input) -> Output {
    state.rule(LifeRestartRule::RangeExact, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_integer(s).and_then(|s| s.tag_node("integer")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}

#[inline]
fn parse_range(state: Input) -> Output {
    state.rule(LifeRestartRule::Range, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_integer(s).and_then(|s| s.tag_node("min"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, ",", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_integer(s).and_then(|s| s.tag_node("max"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}

#[inline]
fn parse_boolean(state: Input) -> Output {
    state.rule(LifeRestartRule::Boolean, |s| {
        Err(s)
            .or_else(|s| builtin_text(s, "true", false).and_then(|s| s.tag_node("true")))
            .or_else(|s| builtin_text(s, "false", false).and_then(|s| s.tag_node("false")))
    })
}

#[inline]
fn parse_kw_attribute(state: Input) -> Output {
    state.rule(LifeRestartRule::KW_ATTRIBUTE, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(属性|属性名)").unwrap())
        })
    })
}

#[inline]
fn parse_kw_trait(state: Input) -> Output {
    state.rule(LifeRestartRule::KW_TRAIT, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(特质|特征)").unwrap())
        })
    })
}

#[inline]
fn parse_kw_event(state: Input) -> Output {
    state.rule(LifeRestartRule::KW_EVENT, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(事件)").unwrap())
        })
    })
}

#[inline]
fn parse_white_space(state: Input) -> Output {
    state.rule(LifeRestartRule::WhiteSpace, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(\\p{White_Space}+)").unwrap())
        })
    })
}

#[inline]
fn parse_comment(state: Input) -> Output {
    state.rule(LifeRestartRule::Comment, |s| {
        Err(s)
            .or_else(|s| {
                builtin_regex(s, {
                    static REGEX: OnceLock<Regex> = OnceLock::new();
                    REGEX.get_or_init(|| Regex::new("^(\\/\\/[^\\n\\r]*)").unwrap())
                })
            })
            .or_else(|s| {
                s.sequence(|s| Ok(s).and_then(|s| builtin_text(s, "/*", false)).and_then(|s| builtin_text(s, "*/", false)))
            })
    })
}

/// All rules ignored in ast mode, inline is not recommended
fn builtin_ignore(state: Input) -> Output {
    state.repeat(0..u32::MAX, |s| parse_white_space(s).or_else(|s| parse_comment(s)))
}

fn builtin_any(state: Input) -> Output {
    state.rule(LifeRestartRule::HiddenText, |s| s.match_char_if(|_| true))
}

fn builtin_text<'i>(state: Input<'i>, text: &'static str, case: bool) -> Output<'i> {
    state.rule(LifeRestartRule::HiddenText, |s| s.match_string(text, case))
}

fn builtin_regex<'i, 'r>(state: Input<'i>, regex: &'r Regex) -> Output<'i> {
    state.rule(LifeRestartRule::HiddenText, |s| s.match_regex(regex))
}
