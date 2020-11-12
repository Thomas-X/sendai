pub mod db {
    use rusqlite::{params, Connection, Row};
    use binance::model::Kline;
    use log::{info, trace, warn};
    use rusqlite::types::ValueRef;

    pub mod wallet {
        use super::*;
        use crate::wallet::wallet::Wallet;
        use std::borrow::Borrow;

        pub fn get_wallets(wallet_conn: &Connection) -> Vec<Wallet> {
            let mut wallet_stmt = wallet_conn.prepare("SELECT * FROM wallet WHERE id = 1").unwrap();
            let wallets = wallet_stmt.query_map(params![], |row| {
                Ok(Wallet {
                    id: row.get(0).unwrap(),
                    balance: row.get(1).unwrap(),
                    last_updated_at: row.get(2).unwrap(),
                })
            })
                .unwrap()
                .map(|f| f.unwrap())
                .collect::<Vec<Wallet>>();
            wallets
        }
    }

    pub mod kline {
        use super::*;

        pub fn create_kline(kline_conn: &Connection, kline: &Kline) {
            kline_conn.execute(
                "REPLACE INTO klines (id, end_time, open, close, high, low, volume, quote_volume) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![kline.start_time, kline.end_time, kline.open, kline.close, kline.high, kline.low, kline.volume, kline.quote_volume],
            ).unwrap();
        }

        pub fn serialize_kline(row: &Row) -> Kline {
            Kline {
                start_time: row.get(0).unwrap(),
                end_time: row.get(1).unwrap(),
                symbol: "".to_string(),
                interval: "".to_string(),
                first_trade_id: 0,
                last_trade_id: 0,
                open: row.get(2).unwrap(),
                close: row.get(3).unwrap(),
                high: row.get(4).unwrap(),
                low: row.get(5).unwrap(),
                volume: row.get(6).unwrap(),
                number_of_trades: 0,
                is_final_bar: false,
                quote_volume: row.get(7).unwrap(),
                active_buy_volume: "".to_string(),
                active_volume_buy_quote: "".to_string(),
                ignore_me: "".to_string(),
            }
        }

        pub fn get_latest_klines(kline_conn: &Connection) -> Vec<Kline> {
            let mut stmt = kline_conn.prepare("SELECT * FROM klines ORDER BY id DESC LIMIT 25").unwrap();
            stmt.query_map(params![], |row| {
                Ok(serialize_kline(row))
            })
                .unwrap()
                .into_iter()
                .map(|f| f.unwrap())
                .collect()
        }

        pub fn create_klines_table(kline_conn: &Connection) {
            kline_conn.execute(
                "CREATE TABLE IF NOT EXISTS klines (
                  id              INTEGER PRIMARY KEY,
                  end_time        INTEGER NOT NULL,
                  open            TEXT NOT NULL,
                  close           TEXT NOT NULL,
                  high            TEXT NOT NULL,
                  low             TEXT NOT NULL,
                  volume          TEXT NOT NULL,
                  quote_volume    TEXT NOT NULL
                  )",
                params![],
            ).unwrap();
        }
    }

    pub mod historical_squeeze {
        use super::*;

        pub struct Squeeze {
            pub value: f64,
            pub price_in_stake: f64,
            pub timestamp: f64,
        }

        pub fn create_squeeze_table(trade_conn: &Connection) {
            trade_conn.execute(
                "CREATE TABLE IF NOT EXISTS squeezes (
                  value           INTEGER NOT NULL,
                  timestamp       INTEGER NOT NULL,
                  price_in_stake  INTEGER NOT NULL
                  )",
                params![],
            ).unwrap();
        }

        pub fn create_squeeze(trade_conn: &Connection, squeeze: &Squeeze) {
            trade_conn.execute(
                "REPLACE INTO squeezes (value, timestamp, price_in_stake) VALUES (?1, ?2, ?3)",
                params![squeeze.value, squeeze.timestamp, squeeze.price_in_stake],
            ).unwrap();
        }

        pub fn get_squeeze_value(trade_conn: &Connection, startup_start_time: f64, avg_exists_from_negative_values: bool) -> (f64, usize) {
            let mut avg_filter = "";
            if avg_exists_from_negative_values {
                avg_filter = " AND value < 0 ";
            } else {
                avg_filter = " AND value > 0 ";
            }
            let mut stmt = trade_conn.prepare(&("SELECT value as avg_value FROM squeezes WHERE timestamp >= ?1".to_owned() + avg_filter + "ORDER BY timestamp desc LIMIT 43200")).unwrap();

            let mut sum: f64 = 0.0;
            let vals = stmt.query_map(params![startup_start_time], |row| {
                Ok(row.get(0).unwrap())
            })
                .unwrap()
                .map(|f| f.unwrap())
                .collect::<Vec<f64>>();
            for val in &vals {
                sum += val.abs()
            }
            return if avg_exists_from_negative_values {
                // 1.1 to be a bit more conservative to buy based on avgs (not 100% avg)
                (-(sum / vals.len() as f64 * 1.1) as f64, vals.len())
            } else {
                ((sum / vals.len() as f64 * 1.1) as f64, vals.len())
            }
        }
    }

    pub mod trade {
        use super::*;

        pub struct Trade {
            pub id: i64,
            pub amount_crypto: String,
            pub amount_money: String,
            pub start_bar_time: i64,
        }

        pub fn create_trades_table(trade_conn: &Connection) {
            trade_conn.execute(
                "CREATE TABLE IF NOT EXISTS trades (
                  id              INTEGER PRIMARY KEY,
                  amount_crypto   TEXT NOT NULL,
                  amount_money    TEXT NOT NULL,
                  start_bar_time  INTEGER NOT NULL
                  )",
                params![],
            ).unwrap();
        }

        pub fn get_trades(trade_conn: &Connection, kline: &Kline, current_bar: bool) -> Vec<Trade> {
            if current_bar {
                let mut stmt = trade_conn.prepare("SELECT * FROM trades WHERE start_bar_time = ?1").unwrap();
                stmt.query_map(params![kline.start_time], |row| {
                    Ok(Trade {
                        id: row.get(0).unwrap(),
                        amount_crypto: row.get(1).unwrap(),
                        amount_money: row.get(2).unwrap(),
                        start_bar_time: row.get(3).unwrap(),
                    })
                })
                    .unwrap()
                    .map(|f| f.unwrap())
                    .collect()
            } else {
                let mut stmt = trade_conn.prepare("SELECT * FROM trades").unwrap();
                stmt.query_map(params![], |row| {
                    Ok(Trade {
                        id: row.get(0).unwrap(),
                        amount_crypto: row.get(1).unwrap(),
                        amount_money: row.get(2).unwrap(),
                        start_bar_time: row.get(3).unwrap(),
                    })
                })
                    .unwrap()
                    .map(|f| f.unwrap())
                    .collect()
            }
        }

        pub fn delete_trade(trade_conn: &Connection, trade_id: i64) {
            trade_conn.execute(
                "DELETE FROM trades WHERE id = ?1",
                params![trade_id],
            ).unwrap();
        }
    }
}
