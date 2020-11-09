use simplelog::{TermLogger, LevelFilter, Config as LogConfig, TerminalMode, ConfigBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::env;
use std::fs;
use log::{info};

#[derive(Clone)]
pub struct Bootstrap {
    pub config: Config
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Pair {
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ApiKey {
    pub key: String,
    pub secret: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub pairs: Vec<Pair>,
    pub api_key: ApiKey,
    pub stake_amount: f64,
    pub min_leftover: f64,
}

impl Bootstrap {

    pub fn new () -> Bootstrap {
        Bootstrap {
            config: Bootstrap::config()
        }
    }

    pub fn boot(&mut self) {
        self.logging();
    }

    pub fn config() -> Config {
        let contents = fs::read_to_string("src/config.json")
            .expect("Couldn't find config file, is it in src/config.json ?");
        let c: Config = serde_json::from_str(&contents)
            .expect("Couldn't read config file, invalid format");
        c
    }

    fn logging(&mut self) {
        let logger_config: LogConfig = ConfigBuilder::new()
            .set_time_format_str("%H:%M:%S:%6f")
            .build();
        TermLogger::init(LevelFilter::Info, logger_config, TerminalMode::Mixed);
    }
}
