use serde::{Serialize, Deserialize};

// TODO: more fields

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalLevelsDB {
    #[serde(rename = "LLM_01")]
    local_levels: Vec<Level>,
    #[serde(rename = "LLM_02")]
    binary_version: i32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Level {
    #[serde(rename = "k1")]
    #[serde(default)]
    id: Option<i32>,
    #[serde(rename = "k2")]
    name: String,
    #[serde(rename = "k5")]
    creator: String
}
