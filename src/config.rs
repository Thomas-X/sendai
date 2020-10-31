pub mod config {
    use serde::{Deserialize, Serialize};
    use serde_json::Result;
    use std::env;
    use std::fs;
    use log::{info};

    #[derive(Serialize, Deserialize)]
    pub struct Pair {
        pub symbol: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ApiKey {
        pub key: String,
        pub secret: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Config {
        pub pairs: Vec<Pair>,
        pub api_key: ApiKey,
    }

    pub fn config() -> Config {
        info!("Reading config");
        let contents = fs::read_to_string("src/config.json")
            .expect("Couldn't find config file, is it in src/config.json ?");
        let c: Config = serde_json::from_str(&contents)
            .expect("Couldn't read config file, invalid format");
        c
    }
}
