use super::*;
use crate::codegen::PropertyStatementNode;

impl LifeVM {
    pub fn load_script(&mut self, text: &str) {
        let ast = RootNode::from_str(text).unwrap();
        for x in ast.statement {
            match x {
                StatementNode::PropertyStatement(v) => self.property.load_property(v),
            }
        }
    }
}

impl PropertyManager {
    fn load_property(&mut self, node: PropertyStatementNode) {
        // node.mat
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
