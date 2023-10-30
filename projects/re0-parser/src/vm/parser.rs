use std::{
    fmt::{Display, Formatter},
    fs::read_to_string,
    path::Path,
};

use url::Url;

use crate::{
    codegen::{
        DescriptionStatementNode, IdStatementNode, PropertyStatementNode, StringNode, TraitItemNode, TraitStatementNode,
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
                StatementNode::TraitStatement(v) => self.talent.load_talent(v, file.clone())?,
                StatementNode::TraitGroup(v) => {
                    for item in v.trait_statement {
                        self.talent.load_talent(item, file.clone())?
                    }
                }
            }
        }
        Ok(())
    }
}

impl PropertyManager {
    fn load_property(&mut self, node: PropertyStatementNode, file: Option<Url>) -> Result<(), LifeError> {
        let mut item = PropertyItem { name: node.identifier.text, file, span: node.identifier.span, ..Default::default() };
        for x in node.property_item {
            match x {
                PropertyItemNode::IdStatement(id) => {
                    item.load_id(id)?;
                }
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
    fn load_talent(&mut self, node: TraitStatementNode, file: Option<Url>) -> Result<(), LifeError> {
        let mut item = TalentItem { name: node.identifier.text, file, span: node.identifier.span, ..Default::default() };
        for x in node.trait_item {
            match x {
                TraitItemNode::DescriptionStatement(v) => item.load_text(v),
                TraitItemNode::IdStatement(v) => item.load_id(v)?,
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
    fn load_id(&mut self, node: IdStatementNode) -> Result<(), LifeError> {
        let id = node.integer.text.parse::<usize>()?;
        self.id = NonZeroUsize::new(id);
        Ok(())
    }
    fn load_text(&mut self, node: DescriptionStatementNode) {
        for x in node.string {
            self.text.push(x.to_string())
        }
    }
}
impl TalentItem {
    fn load_id(&mut self, node: IdStatementNode) -> Result<(), LifeError> {
        let id = node.integer.text.parse::<usize>()?;
        self.id = NonZeroUsize::new(id);
        Ok(())
    }
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

fn test_classes() {
    let text = r##"text class RegexInner {
    /([^\\\\\\/]|\\\\.)+/
}"##;
    let cst = LifeRestartParser::parse_cst(text, LifeRestartRule::Root).unwrap();
    println!("Short Form:\n{}", cst);
    let ast = RootNode::from_str(text).unwrap();
    println!("{ast:#?}")
}
