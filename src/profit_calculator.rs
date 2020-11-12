pub mod profit_calculator {
    use crate::bootstrap::{Pair, Bootstrap, Config};
    use rusqlite::Connection;
    use std::{thread, time};
    use crate::db::db::trade::{Trade, get_trades, delete_trade};
    use binance::model::Kline;
    use crate::db::db::kline::get_latest_kline;
    use log::{info, trace, warn};
    use binance::api::Binance;
    use binance::account::Account;
    use std::borrow::Borrow;

    pub fn run(config: Config) {
        let pairs = &config.pairs;
        let mut connections = vec![];
        for pair in pairs {
            let trade_conn = Connection::open(format!("trades-{}.db", &pair.symbol)).unwrap();
            let kline_conn = Connection::open(format!("{}.db", &pair.symbol)).unwrap();
            connections.push((trade_conn, kline_conn, format!("trades-{}.db", &pair.symbol)));
        }
        let quote_order_qty = config.stake_amount;
        loop {
            let mut profit: f64 = 0.0;
            let mut total_trades_on_the_line: f64 = 0.0;
            for (trade_conn, kline_conn, _) in &connections {
                let trades = get_trades(&trade_conn, &Kline {
                    start_time: 0,
                    end_time: 0,
                    symbol: "".to_string(),
                    interval: "".to_string(),
                    first_trade_id: 0,
                    last_trade_id: 0,
                    open: "".to_string(),
                    close: "".to_string(),
                    high: "".to_string(),
                    low: "".to_string(),
                    volume: "".to_string(),
                    number_of_trades: 0,
                    is_final_bar: false,
                    quote_volume: "".to_string(),
                    active_buy_volume: "".to_string(),
                    active_volume_buy_quote: "".to_string(),
                    ignore_me: "".to_string(),
                }, false);
                for trade in &trades {
                    let qty = trade.amount_crypto.parse::<f64>().unwrap();
                    let kline = &get_latest_kline(&kline_conn)[0];
                    let diff = &kline.close.parse::<f64>().unwrap() * qty - quote_order_qty;
                    profit += diff as f64;
                    total_trades_on_the_line += 1.0;
                }
                // fees for buy&sell (0.00075 * 2) 2 = buying and selling
                profit -= ((quote_order_qty * 0.0015) * trades.len() as f64) as f64
            }

            let mut sell_handles = vec![];
            info!("\n\n current total profit is: {:?} \n", profit);
            // profit is higher or equal to 1% of total stake amount (trades on the line
            if profit >= ((total_trades_on_the_line * quote_order_qty) as f64 * 0.01) {
                // sell all
                for (trade_conn, kline_conn, trade_conn_string) in &connections {
                    let trades = get_trades(&trade_conn, &Kline {
                        start_time: 0,
                        end_time: 0,
                        symbol: "".to_string(),
                        interval: "".to_string(),
                        first_trade_id: 0,
                        last_trade_id: 0,
                        open: "".to_string(),
                        close: "".to_string(),
                        high: "".to_string(),
                        low: "".to_string(),
                        volume: "".to_string(),
                        number_of_trades: 0,
                        is_final_bar: false,
                        quote_volume: "".to_string(),
                        active_buy_volume: "".to_string(),
                        active_volume_buy_quote: "".to_string(),
                        ignore_me: "".to_string(),
                    }, false);
                    for trade in trades {
                        let qty = trade.amount_crypto.parse::<f64>().unwrap();
                        let kline = &get_latest_kline(&kline_conn)[0];
                        let diff = &kline.close.parse::<f64>().unwrap() * &qty - quote_order_qty;
                        let api_key = config.api_key.key.clone();
                        let api_key_secret = config.api_key.secret.clone();
                        let trade_conn_string_clone = trade_conn_string.clone();
                        let symbol = kline.symbol.clone();
                        let sell_handle = thread::spawn(move || {
                            // ugly!
                            let account: Account = Binance::new(Some(api_key), Some(api_key_secret));
                            let trade_conn_copy = Connection::open(trade_conn_string_clone).unwrap();

                            match account.market_sell::<&str, f64>(symbol.borrow(), qty) {
                                Ok(e) => {
                                    delete_trade(&trade_conn_copy, trade.id);
                                    info!("Sold crypto at profit of, {:?} USDT, {:?}", diff, e)
                                }
                                Err(e) => warn!("Couldn't sell because error: {:?}", e)
                            }
                        });
                        sell_handles.push(sell_handle);
                    }
                }
            }
            // wait for all sells to finish
            for sell_handle in sell_handles {
                sell_handle.join().unwrap();
            }
            thread::sleep(time::Duration::from_secs(10));
        }
    }
}
