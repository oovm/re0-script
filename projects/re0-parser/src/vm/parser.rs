use std::{fs::read_to_string, path::Path};

use url::Url;

use crate::{
    codegen::{IdStatementNode, PropertyStatementNode},
    LifeError,
};

use super::*;

impl LifeVM {
    pub fn load_script(&mut self, text: &str) -> Result<(), LifeError> {
        let ast = RootNode::from_str(text).unwrap();
        for x in ast.statement {
            match x {
                StatementNode::PropertyStatement(v) => self.property.load_property(v, None)?,
            }
        }
        Ok(())
    }
    pub fn load_local(&mut self, file: &Path) -> Result<(), LifeError> {
        let url = Url::from_file_path(file.canonicalize().unwrap()).unwrap();
        let text = read_to_string(file).unwrap();
        let ast = RootNode::from_str(&text).unwrap();
        for x in ast.statement {
            match x {
                StatementNode::PropertyStatement(v) => self.property.load_property(v, Some(url.clone()))?,
            }
        }
        Ok(())
    }
}

impl PropertyManager {
    fn load_property(&mut self, node: PropertyStatementNode, file: Option<Url>) -> Result<(), LifeError> {
        let mut item = PropertyItem { name: node.identifier.text, file, span: node.identifier.span, ..PropertyItem::default() };
        for x in node.property_item {
            match x {
                PropertyItemNode::IdStatement(id) => {
                    item.load_id(id)?;
                }
            }
        }
        self.insert(item)?;
        Ok(())
        // node.mat
    }
}

impl PropertyItem {
    fn load_id(&mut self, id: IdStatementNode) -> Result<(), LifeError> {
        let id = id.integer.text.parse::<usize>()?;
        self.id = NonZeroUsize::new(id);
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
