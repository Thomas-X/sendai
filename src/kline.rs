
pub mod kline {
    extern crate binance;

    use binance::websockets::*;
    use std::sync::atomic::AtomicBool;
    use self::binance::model::{KlineEvent, KlineSummaries, Kline};
    use log::{info, trace, warn};
    use rusqlite::{params, Connection, Result};
    use crate::config::config::{config, Config};
    use binance::market::*;
    use self::binance::api::Binance;
    use self::binance::errors::Error;

    pub type KlineHandler = fn(KlineEvent, &Connection) -> ();

    pub fn handle_kline_event(kline_event: KlineEvent, conn: &Connection) {
        let kline = &kline_event.kline;
        conn.execute(
            "REPLACE INTO klines (id, end_time, open, close, high, low, volume, quote_volume) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![kline.start_time, kline.end_time, kline.open, kline.close, kline.high, kline.low, kline.volume, kline.quote_volume],
        ).unwrap();
        // this means it's a "real" event, not from fillup and we should act
        if kline.symbol != "" {
            let mut stmt = conn.prepare("SELECT * FROM klines ORDER BY id DESC LIMIT 500").unwrap();
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
                    ignore_me: "".to_string()
                })
            }).unwrap();

            // for kline in kline_iter {
            //     println!("Found kline {:?}", kline.unwrap());
            // }

            // then feed this into a "strategy" and act upon it's orders

            info!("{} latest price: {}", kline.symbol, kline.close)
        }
    }

    pub fn kline_data_fillup(symbol: &String, config: Config, conn: &Connection) {
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
                                    ignore_me: "".to_string()
                                }
                            };
                            handle_kline_event(k, &conn)
                        }
                    }
                }
            }
            Err(error) => warn!("Could not get past klines {}", error)
        };
        info!("Successfully downloaded and saved all past 500 klines")
    }

    pub fn open_kline_stream(handler: KlineHandler, symbol: String, conn: Connection) {
        let config = config();
        kline_data_fillup(&symbol, config, &conn);
        let keep_running = AtomicBool::new(true);
        let kline: String = format!("{}", format!("{}@kline_1m", symbol.to_lowercase()));
        let mut web_socket: WebSockets = WebSockets::new(|event: WebsocketEvent| {
            match event {
                WebsocketEvent::Kline(kline_event) => handler(kline_event, &conn),
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
