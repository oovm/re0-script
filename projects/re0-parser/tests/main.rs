#[test]
fn ready() {
    println!("it works!")
}

#[test]
fn test_classes() {
    let text = r##"text class RegexInner {
    /([^\\\\\\/]|\\\\.)+/
}"##;
    let cst = Parser::parse_cst(text, BootstrapRule::ClassStatement).unwrap();
    println!("Short Form:\n{}", cst);
    let ast = ClassStatementNode::from_str(text).unwrap();
    println!("{ast:#?}")
}
