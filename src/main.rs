// This file is temporary

use gdparser_temp::serde as gd_serde;
use gdparser_temp::local_levels::LocalLevelsDB;
use gdparser_temp::game_manager::GameManagerDB;
use std::{env, path::PathBuf, fs::File};

fn main() {
    let mut path1 = PathBuf::from(env::var("localappdata").unwrap());
    let mut path2 = path1.clone();
    path1.push("GeometryDash/CCLocalLevels.dat");
    path2.push("GeometryDash/CCGameManager.dat");
    
    let file1 = File::open(path1).unwrap();
    let file2 = File::open(path2).unwrap();

    // Showing content of CCLocalLevels.dat just to show effects of deserializing
    // It'll be replaced with unit/integrated tests later when the project grew

    let mut x = gd_serde::de::Deserializer::from_reader(file1).unwrap();
    x._test();

    // let ldb: LocalLevelsDB = gd_serde::from_reader(file1).unwrap();
    // print!("{ldb:#?}");
    let gdb: gd_serde::de::DataWithHeader<GameManagerDB> = gd_serde::from_reader(file2).unwrap();
    print!("{:#?}", gdb);
}
