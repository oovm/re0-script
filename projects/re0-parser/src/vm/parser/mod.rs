#![allow(clippy::wrong_self_convention)]

use pratt::{Affix, Associativity, PrattParser, Precedence};
use std::{
    fmt::{Display, Formatter},
    fs::read_to_string,
    path::Path,
    vec::IntoIter,
};

use url::Url;

use crate::{
    codegen::{
        AtomicNode, BooleanNode, DescriptionStatementNode, EventGroupNode, EventItemNode, EventStatementNode, ExpressionNode,
        IdStatementNode, IdentifierNode, InfixNode, PrefixNode, PropertyStatementNode, StringNode, SuffixNode, TermNode,
        TopStatementNode, TraitItemNode, TraitStatementNode,
    },
    vm::talents::TalentItem,
    LifeError,
};

use super::*;

impl LifeVM {
    pub fn load_script(&mut self, text: &str) -> Result<(), LifeError> {
        let ast = RootNode::from_str(text).unwrap();
        self.load_statements(ast, None)?;
        Ok(())
    }
    pub fn load_local<P>(&mut self, file: P) -> Result<(), LifeError>
    where
        P: AsRef<Path>,
    {
        let file = file.as_ref().canonicalize()?;
        let url = Url::from_file_path(&file)?;
        let text = read_to_string(&file)?;
        let ast = RootNode::from_str(&text)?;
        self.load_statements(ast, Some(url))?;
        Ok(())
    }
    fn load_statements(&mut self, root: RootNode, file: Option<Url>) -> Result<(), LifeError> {
        for x in root.top_statement {
            match x {
                TopStatementNode::PropertyStatement(v) => self.property.load_property(v, file.clone())?,
                TopStatementNode::TraitStatement(v) => self.talent.load_talent(v, &file)?,
                TopStatementNode::TraitGroup(v) => {
                    for item in v.trait_statement {
                        self.talent.load_talent(item, &file)?
                    }
                }
                TopStatementNode::EventGroup(v) => {}
                TopStatementNode::EventStatement(v) => self.story.insert(v.as_story(&file)?)?,
                TopStatementNode::Eos(_) => {}
            }
        }
        Ok(())
    }
}

impl EventGroupNode {
    pub fn as_story(self, file: &Option<Url>) -> Result<StoryItem, LifeError> {
        let mut out = StoryItem::new(self.identifier.as_identifier(file));
        // for item in self.event_statement {
        //     item.mat
        // }
        Ok(out)
    }
}
impl EventStatementNode {
    pub fn as_story(self, file: &Option<Url>) -> Result<StoryItem, LifeError> {
        let mut out = StoryItem::new(self.identifier.as_identifier(file));
        for item in self.event_item {
            match item {
                EventItemNode::DescriptionStatement(_) => {}
                EventItemNode::IdStatement(v) => {}
                EventItemNode::OptionStatement(_) => {}
                EventItemNode::RequirementStatement(_) => {}
            }
        }
        Ok(out)
    }
}
impl IdentifierNode {
    pub fn as_identifier(self, file: &Option<Url>) -> Identifier {
        Identifier { name: self.text, file: file.clone(), span: self.span }
    }
}

impl PropertyManager {
    fn load_property(&mut self, node: PropertyStatementNode, file: Option<Url>) -> Result<(), LifeError> {
        let mut item = PropertyItem::new(Identifier { name: node.identifier.text, file, span: node.identifier.span });
        for x in node.property_item {
            match x {
                PropertyItemNode::IdStatement(v) => item.index = v.as_index().ok(),
                PropertyItemNode::DescriptionStatement(v) => item.load_text(v),
                PropertyItemNode::RequirementStatement(_) => {}
                PropertyItemNode::EnumerateStatement(_) => {}
                PropertyItemNode::OptionStatement(_) => {}
                PropertyItemNode::Eos(_) => {}
            }
        }
        self.insert(item)?;
        Ok(())
        // node.mat
    }
}
impl TalentManager {
    fn load_talent(&mut self, node: TraitStatementNode, file: &Option<Url>) -> Result<(), LifeError> {
        let mut item = TalentItem::new(node.identifier.as_identifier(file));
        for x in node.trait_item {
            match x {
                TraitItemNode::DescriptionStatement(v) => item.load_text(v),
                TraitItemNode::IdStatement(v) => item.index = v.as_index().ok(),
                TraitItemNode::RequirementStatement(_) => {}
                TraitItemNode::EffectStatement(_) => {}
            }
        }
        self.insert(item)?;
        Ok(())
        // node.mat
    }
}

impl PropertyItem {
    fn load_text(&mut self, node: DescriptionStatementNode) {
        for x in node.string {
            self.text.push(x.to_string())
        }
    }
}
impl IdStatementNode {
    pub fn as_index(&self) -> Result<NonZeroUsize, LifeError> {
        let id = self.integer.text.parse::<usize>()?;
        Ok(NonZeroUsize::new(id).unwrap())
    }
}

impl TalentItem {
    fn load_text(&mut self, node: DescriptionStatementNode) {
        for x in node.string {
            self.text.push(x.to_string())
        }
    }
}

#[derive(Debug)]
enum Expr {
    Term(AtomicNode),
    Prefix(PrefixNode),
    Infix(InfixNode),
    Suffix(SuffixNode),
}

impl PrattParser<IntoIter<Expr>> for Expression {
    type Error = LifeError;
    type Input = Expr;
    type Output = Expression;

    fn query(&mut self, input: &Self::Input) -> Result<Affix, Self::Error> {
        let affix = match input {
            Expr::Term(_) => Affix::Nilfix,
            Expr::Prefix(_) => Affix::Prefix(Precedence(2000)),
            Expr::Infix(o) => match o {
                InfixNode::Add => Affix::Infix(Precedence(100), Associativity::Left),
                InfixNode::Sub => Affix::Infix(Precedence(100), Associativity::Left),
                InfixNode::AddAssign => Affix::Infix(Precedence(90), Associativity::Right),
                InfixNode::SubAssign => Affix::Infix(Precedence(90), Associativity::Right),
                InfixNode::And => Affix::Infix(Precedence(2), Associativity::Left),
                InfixNode::EQ => Affix::Infix(Precedence(2), Associativity::Left),
                InfixNode::GEQ => Affix::Infix(Precedence(2), Associativity::Left),
                InfixNode::GT => Affix::Infix(Precedence(2), Associativity::Left),
                InfixNode::Has => Affix::Infix(Precedence(2), Associativity::Left),
                InfixNode::LEQ => Affix::Infix(Precedence(2), Associativity::Left),
                InfixNode::LT => Affix::Infix(Precedence(2), Associativity::Left),
                InfixNode::NE => Affix::Infix(Precedence(2), Associativity::Left),
                InfixNode::Or => Affix::Infix(Precedence(2), Associativity::Left),
            },
            Expr::Suffix(_) => Affix::Postfix(Precedence(3000)),
        };
        Ok(affix)
    }

    fn primary(&mut self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let expr = match input {
            Expr::Term(v) => match v {
                AtomicNode::Boolean(v) => match v {
                    BooleanNode::False => Expression::Boolean(false),
                    BooleanNode::True => Expression::Boolean(true),
                },
                AtomicNode::Identifier(_) => Expression::Boolean(true),
                AtomicNode::Integer(_) => Expression::Boolean(true),
                AtomicNode::List(_) => Expression::Boolean(true),
                AtomicNode::StringNormal(_) => Expression::Boolean(true),
                AtomicNode::StringRaw(_) => Expression::Boolean(true),
            },
            _ => unreachable!(),
        };
        Ok(expr)
    }

    fn infix(&mut self, lhs: Self::Output, tree: Self::Input, rhs: Self::Output) -> Result<Self::Output, Self::Error> {
        let o = match tree {
            Expr::Infix(v) => match v {
                InfixNode::Add => Operator::Add,
                InfixNode::AddAssign => Operator::Add,
                InfixNode::And => Operator::Add,
                InfixNode::EQ => Operator::Add,
                InfixNode::GEQ => Operator::Add,
                InfixNode::GT => Operator::Add,
                InfixNode::Has => Operator::Add,
                InfixNode::LEQ => Operator::Add,
                InfixNode::LT => Operator::Add,
                InfixNode::NE => Operator::Add,
                InfixNode::Or => Operator::Add,
                InfixNode::Sub => Operator::Add,
                InfixNode::SubAssign => Operator::Add,
            },
            _ => unreachable!(),
        };
        Ok(Expression::infix(lhs, o, rhs))
    }

    fn prefix(&mut self, tree: Self::Input, rhs: Self::Output) -> Result<Self::Output, Self::Error> {
        let o = match tree {
            Expr::Prefix(v) => match v {
                PrefixNode::Prefix0 => Operator::Add,
            },
            _ => unreachable!(),
        };
        Ok(Expression::prefix(o, rhs))
    }

    fn postfix(&mut self, lhs: Self::Output, tree: Self::Input) -> Result<Self::Output, Self::Error> {
        let o = match tree {
            Expr::Suffix(v) => match v {
                SuffixNode::Suffix0 => Operator::Add,
            },
            _ => unreachable!(),
        };
        Ok(Expression::suffix(lhs, o))
    }
}

impl Display for StringNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StringNode::StringRaw(r) => f.write_str(&r.string_raw_text.text)?,
        }
        Ok(())
    }
}
