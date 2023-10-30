use std::str::FromStr;

use yggdrasil_rt::YggdrasilParser;

use re0_parser::codegen::{LifeRestartParser, LifeRestartRule, RootNode};

#[test]
fn ready() {
    println!("it works!")
}

#[test]
fn test_classes() {
    let text = r##"text class RegexInner {
    /([^\\\\\\/]|\\\\.)+/
}"##;
    let cst = LifeRestartParser::parse_cst(text, LifeRestartRule::Root).unwrap();
    println!("Short Form:\n{}", cst);
    let ast = RootNode::from_str(text).unwrap();
    println!("{ast:#?}")
}
