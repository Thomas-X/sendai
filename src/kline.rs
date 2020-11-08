pub mod kline {
    extern crate binance;

    use binance::websockets::*;
    use std::sync::atomic::AtomicBool;
    use self::binance::model::{KlineEvent, KlineSummaries, Kline};
    use log::{info, trace, warn};
    use rusqlite::{params, Connection, Result, Statement};
    use crate::config::config::{config, Config};
    use binance::market::*;
    use self::binance::api::Binance;
    use self::binance::errors::Error;
    use crate::strategy::strategy;
    use self::binance::account::Account;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use crate::wallet::wallet::Wallet;

    pub type KlineHandler = fn(KlineEvent, &Connection, &Connection, &Connection) -> ();


    pub struct Trade {
        pub id: i64,
        pub amount_crypto: String,
        pub amount_money: String,
        pub start_bar_time: i64,
    }

    pub fn get_trades (trade_conn: &Connection, kline: &Kline, current_bar: bool) -> Vec<Trade> {
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

    pub fn handle_kline_event(kline_event: KlineEvent, conn: &Connection, wallet_conn: &Connection, trade_conn: &Connection) {
        let kline = &kline_event.kline;
        conn.execute(
            "REPLACE INTO klines (id, end_time, open, close, high, low, volume, quote_volume) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![kline.start_time, kline.end_time, kline.open, kline.close, kline.high, kline.low, kline.volume, kline.quote_volume],
        ).unwrap();
        // this means it's a "real" event, not from fillup and we should act
        if kline.symbol != "" {
            let config = config();
            let mut stmt = conn.prepare("SELECT * FROM klines ORDER BY id DESC LIMIT 25").unwrap();
            let kline_iter = stmt.query_map(params![], |row| {
                Ok(Kline {
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
                })
            }).unwrap();
            let mut klines = vec![];
            for kline in kline_iter {
                klines.push(kline.unwrap());
            }

            let (should_sell, should_buy) = strategy::calculate(&klines, &trade_conn);

            let account: Account = Binance::new(Option::from(config.api_key.key), Option::from(config.api_key.secret));
            let quote_order_qty = 11.0;

            if get_trades(&trade_conn, &kline, true).len() < 1 {
                let mut wallet_stmt = wallet_conn.prepare("SELECT * FROM wallet WHERE id = 1").unwrap();
                let wallets = wallet_stmt.query_map(params![], |row| {
                    Ok(Wallet {
                        id: row.get(0).unwrap(),
                        balance: row.get(1).unwrap(),
                        last_updated_at: row.get(2).unwrap()
                    })
                })
                    .unwrap()
                    .map(|f| f.unwrap())
                    .collect::<Vec<Wallet>>();
                let wallet = wallets.first().unwrap();
                // we're out of $$$ to buy, lets stop
                if wallet.balance.parse::<f64>().unwrap() < (quote_order_qty * 2.0) {
                    info!("didn't buy because wallet balance is too low (this could be just because we have a lot of trades open too)")
                } else if should_buy {
                    // 11 USDT
                    match account.market_buy::<&str, f64>(&kline.symbol, quote_order_qty) {
                        Ok(answer) => {
                            info!("Bought {} at {}, amount: {}", &kline.symbol, answer.price, answer.executed_qty);
                            trade_conn.execute(
                                "INSERT INTO trades (id, amount_crypto, amount_money, start_bar_time) VALUES (?1, ?2, ?3, ?4)",
                                params![answer.order_id as i64, answer.executed_qty, quote_order_qty, &kline.start_time],
                            ).unwrap();
                        }
                        Err(e) => warn!("Error: {:?}", e),
                    }
                }
            }

            for trade in get_trades(&trade_conn, &kline, false) {
                let qty = trade.amount_crypto.parse::<f64>().unwrap();
                let diff = &kline.close.parse::<f64>().unwrap() * qty - quote_order_qty;
                info!("diff, value to break even: {:?} {:?}", diff, quote_order_qty * 0.015);
                // 0.075% for buying, 0.075% fee for selling, if we are above that we made a profit
                if diff > (quote_order_qty * 0.015) && should_sell {
                    // sell, we have made profit
                    match account.market_sell::<&str, f64>(&kline.symbol, qty) {
                        Ok(e) => {
                            delete_trade(&trade_conn, trade.id);
                            info!("Sold crypto at profit of, {:?} USDT, {:?}", diff, e)
                        },
                        Err(e) => warn!("Couldn't sell because error: {:?}", e)
                    }
                    // -3 <= -0.65
                } else if diff <= -(quote_order_qty * 0.2) {
                    // sell, we have made loss at -5% stoploss
                    match account.market_sell::<&str, f64>(&kline.symbol, qty) {
                        Ok(e) => {
                            delete_trade(&trade_conn, trade.id);
                            info!("Sold crypto at 5% LOSS, {:?}", e)
                        },
                        Err(e) => warn!("Couldn't sell because error: {:?}", e)
                    }
                }
            }
        }
    }

    pub fn delete_trade(trade_conn: &Connection, trade_id: i64) {
        trade_conn.execute(
            "DELETE FROM trades WHERE id = ?1",
            params![trade_id],
        ).unwrap();
    }

    pub fn kline_data_fillup(symbol: &String, config: Config, conn: &Connection, wallet_conn: &Connection, trade_conn: &Connection) {
        info!("Doing data fillup of past 500 klines");
        let market: Market = Binance::new(None, None);
        match market.get_klines(symbol, "1m", 500, None, None) {
            Ok(kline_summaries) => {
                match kline_summaries {
                    KlineSummaries::AllKlineSummaries(klines) => {
                        for kline in klines {
                            let k = KlineEvent {
                                event_type: "".to_string(),
                                event_time: 0,
                                symbol: "".to_string(),
                                kline: Kline {
                                    start_time: kline.open_time,
                                    end_time: kline.close_time,
                                    symbol: "".to_string(),
                                    interval: "".to_string(),
                                    first_trade_id: 0,
                                    last_trade_id: 0,
                                    open: kline.open.to_string(),
                                    close: kline.close.to_string(),
                                    high: kline.high.to_string(),
                                    low: kline.low.to_string(),
                                    volume: kline.volume.to_string(),
                                    number_of_trades: 0,
                                    is_final_bar: false,
                                    quote_volume: kline.quote_asset_volume.to_string(),
                                    active_buy_volume: "".to_string(),
                                    active_volume_buy_quote: "".to_string(),
                                    ignore_me: "".to_string(),
                                },
                            };
                            handle_kline_event(k, &conn, &wallet_conn, &trade_conn);
                        }
                    }
                }
            }
            Err(error) => warn!("Could not get past klines {}", error)
        };
        info!("Successfully downloaded and saved all needed past klines")
    }



    pub fn open_kline_stream(handler: KlineHandler, symbol: String, conn: Connection, wallet_conn: Connection, trade_conn: Connection) {
        trade_conn.execute(
            "CREATE TABLE IF NOT EXISTS trades (
                  id              INTEGER PRIMARY KEY,
                  amount_crypto            TEXT NOT NULL,
                  amount_money           TEXT NOT NULL,
                  start_bar_time    INTEGER NOT NULL
                  )",
            params![],
        ).unwrap();
        let config = config();
        kline_data_fillup(&symbol, config, &conn, &wallet_conn, &trade_conn);
        let keep_running = AtomicBool::new(true);
        let kline: String = format!("{}", format!("{}@kline_1m", symbol.to_lowercase()));
        let mut web_socket: WebSockets = WebSockets::new(|event: WebsocketEvent| {
            match event {
                WebsocketEvent::Kline(kline_event) => {
                    handler(kline_event, &conn, &wallet_conn, &trade_conn)
                },
                _ => ()
            };
            Ok(())
        });
        web_socket.connect(&kline).unwrap(); // check error
        if let Err(e) = web_socket.event_loop(&keep_running) {
            match e {
                err => {
                    warn!("Error with websocket event loop {}", err);
                }
            }
        }
        web_socket.disconnect().unwrap();
    }
}
