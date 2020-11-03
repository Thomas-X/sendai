pub mod wallet {
    use binance::account::*;
    use binance::api::Binance;
    use crate::config::config::{Config, config};
    use std::thread;
    use core::time;
    use log::{info, trace, warn};
    use rusqlite::{params, Connection, Result};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub struct Wallet {
        pub id: i64,
        pub balance: String,
        pub last_updated_at: i64,
    }

    pub fn refresh() {
        let config = config();
        let wallet_conn = Connection::open("wallet.db").unwrap();
        let account: Account = Binance::new(Option::from(config.api_key.key), Option::from(config.api_key.secret));
        let wallet_refresh_time = 10000;
        wallet_conn.execute(
            "CREATE TABLE IF NOT EXISTS wallet (
                      id              INTEGER PRIMARY KEY,
                      balance         TEXT NOT NULL,
                      last_updated_at INTEGER NOT NULL
                      )",
            params![],
        ).unwrap();
        loop {
            let account_information = account.get_account();
            if account_information.is_err() {
                warn!("err when updating wallet {:?}", account_information.err().unwrap());
                continue;
            }
            let v = account_information.ok().unwrap();
            let balance = v.balances
                .iter()
                .find(|f| f.asset == "USDT")
                .unwrap();
            wallet_conn.execute(
                "REPLACE INTO wallet (id, balance, last_updated_at) VALUES (?1, ?2, ?3)",
                params![1, balance.free, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64],
            ).unwrap();
            info!("Refreshed wallet");
            // wait till next check
            thread::sleep(time::Duration::from_millis(wallet_refresh_time));
        }
    }
}
