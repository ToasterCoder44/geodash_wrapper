// This file is temporary

use gdparser_temp::serde as gd_serde;
use serde::{Deserialize};
use std::{env, path::PathBuf, fs::File};

fn main() {
    let mut path = PathBuf::from(env::var("localappdata").unwrap());
    path.push("GeometryDash/CCLocalLevels.dat");
    
    let file = File::open(path).unwrap();
    let mut deserializer = gd_serde::from_reader(file).unwrap();

    // Showing content of CCLocalLevels.dat just to show effects of deserializing
    // It'll be replaced with unit/integrated tests later when the project grew

    let ldb = gdparser_temp::local_levels::LocalLevelsDB::deserialize(&mut deserializer);
    print!("{ldb:#?}");
}
