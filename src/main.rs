// This file is temporary

use gdparser_temp::serde as gd_serde;
use gdparser_temp::local_levels::LocalLevelsDB;
use gdparser_temp::game_manager::GameManagerDB;
use gd_serde::de::DataWithHeader;
use serde::Serialize;
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

    let ldb: DataWithHeader<LocalLevelsDB> = gd_serde::from_reader(file1).unwrap();
    println!("{ldb:#?}");
    let gdb: DataWithHeader<GameManagerDB> = gd_serde::from_reader(file2).unwrap();
    println!("{:#?}", gdb);

    let mut ser = gd_serde::ser::Serializer::new();
    (5346.32357328946).serialize(&mut ser).unwrap();
    println!("{}", String::from_utf8(ser.writer.into_inner().into_inner()).unwrap());
}
