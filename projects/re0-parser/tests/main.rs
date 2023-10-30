use std::path::Path;

use re0_parser::vm::LifeVM;

#[test]
fn ready() {
    println!("it works!")
}

#[test]
fn test_vm() {
    let mut vm = LifeVM::default();
    vm.load_local(&Path::new("tests/属性.re0")).unwrap();
    println!("{vm:#?}")
}

// #[test]
// fn test_options() {
//     let text = r##"选项[]"##;
//     let cst = LifeRestartParser::parse_cst(text, LifeRestartRule::OptionStatement).unwrap();
//     println!("Short Form:\n{}", cst);
//     let ast = OptionStatementNode::from_str(text).unwrap();
//     println!("{ast:#?}")
// }
