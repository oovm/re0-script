#![allow(clippy::wrong_self_convention)]

use std::{
    fmt::{Display, Formatter},
    fs::read_to_string,
    path::Path,
};

use url::Url;

use crate::{
    codegen::{
        DescriptionStatementNode, EventGroupNode, EventItemNode, EventStatementNode, IdStatementNode, IdentifierNode,
        PropertyStatementNode, StringNode, TraitItemNode, TraitStatementNode,
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
        let url = Url::from_file_path(&file).unwrap();
        let text = read_to_string(&file)?;
        let ast = RootNode::from_str(&text).unwrap();
        self.load_statements(ast, Some(url))?;
        Ok(())
    }
    fn load_statements(&mut self, root: RootNode, file: Option<Url>) -> Result<(), LifeError> {
        for x in root.statement {
            match x {
                StatementNode::PropertyStatement(v) => self.property.load_property(v, file.clone())?,
                StatementNode::TraitStatement(v) => self.talent.load_talent(v, &file)?,
                StatementNode::TraitGroup(v) => {
                    for item in v.trait_statement {
                        self.talent.load_talent(item, &file)?
                    }
                }
                StatementNode::EventGroup(v) => {}
                StatementNode::EventStatement(v) => self.story.insert(v.as_story(&file)?)?,
                StatementNode::Eos(_) => {}
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
                EventItemNode::Eos(_) => {}
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
                TraitItemNode::Eos(_) => {}
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
impl Display for StringNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StringNode::StringRaw(r) => f.write_str(&r.string_raw_text.text)?,
        }
        Ok(())
    }
}
