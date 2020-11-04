mod config;
mod logging;
mod kline;
mod strategy;
mod squeeze_momentum;
mod sma;
mod stdev;
mod trange;
mod linreg;
mod highest;
mod lowest;
mod avg;
mod wallet;

use std::sync::atomic::{AtomicBool};
use log::{info, trace, warn};
use std::sync::{Mutex, Arc};
use std::rc::Rc;
use std::thread;
use rusqlite::{params, Connection, Result};
use crate::kline::kline::*;
use crate::config::config::config;
use crate::wallet::wallet::refresh;

fn main() -> () {
    logging::logging();
    info!("Starting up");
    let config = config();
    let pairs_len = config.pairs.len();
    let pairs = Arc::new(Mutex::new(config.pairs));
    let mut handles = vec![];

    // Wallet synchronization
    thread::spawn(refresh);

    for i in 0..pairs_len {
        let pairs = Arc::clone(&pairs);
        let handle = thread::spawn(move || {
            let mut pairs_lock = pairs.lock().unwrap();
            let symbol = pairs_lock[i].symbol.clone();
            let conn = Connection::open(format!("{}.db", symbol)).unwrap();
            let wallet_conn = Connection::open("wallet.db").unwrap();
            let trade_conn = Connection::open(format!("trades-{}.db", symbol)).unwrap();
            conn.execute(
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
            info!("Opening stream for {}", symbol);
            // we can't rely on Rust's out-of-scope dropping of lock because we're doing a sync call that never makes the lock go out of scope.
            // we know what we're doing though, the compiler doesn't
            drop(pairs_lock);
            open_kline_stream(handle_kline_event, symbol, conn, wallet_conn, trade_conn);
        });
        handles.push(handle)
    }
    for handle in handles {
        handle.join().unwrap();
    }
}
