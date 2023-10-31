use re0_parser::LifeError;
use std::path::Path;

use re0_parser::vm::LifeVM;

#[test]
fn ready() {
    println!("it works!")
}

#[test]
fn test_vm() {
    let mut vm = LifeVM::default();
    vm.load_local("tests/属性.re0").unwrap();
    println!("{vm:#?}")
}

#[test]
fn test_dir() {
    let mut vm = LifeVM::default();
    let glob = globwalk::GlobWalkerBuilder::new(r#"C:\P4Root\project\DatingSimulator\DataTables\Life"#, "*.{re0,life}")
        .build()
        .unwrap();
    for img in glob {
        if let Ok(img) = img {
            match vm.load_local(img.path()) {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", img.path().display());
                    println!("{:?}", e);
                }
            }
        }
    }
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
