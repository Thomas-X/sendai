use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::env;
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct Pair {
    symbol: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pairs: Vec<Pair>,
}


pub(crate) fn config() -> Result<()> {
    let contents = fs::read_to_string("src/config.json")
        .expect("Couldn't find config file, is it in src/config.json ?");
    let c: Config = serde_json::from_str(&contents)?;


    Ok(())
}
