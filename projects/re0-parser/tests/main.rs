use std::{path::Path, str::FromStr};

use yggdrasil_rt::YggdrasilParser;

use re0_parser::vm::LifeVM;

#[test]
fn ready() {
    println!("it works!")
}

#[test]
fn test_classes() {
    let mut vm = LifeVM::default();
    vm.load_local(&Path::new("tests/属性.re0"));
    println!("{vm:#?}")
}
