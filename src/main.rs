mod kline;
mod squeeze_momentum;
mod wallet;
mod indicators;
mod bootstrap;
mod db;
mod strategy;
mod util;
mod profit_calculator;

use std::sync::atomic::{AtomicBool};
use log::{info, trace, warn};
use std::sync::{Mutex, Arc};
use std::rc::Rc;
use std::thread;
use rusqlite::{params, Connection, Result};
use crate::kline::kline::*;
use crate::wallet::wallet::refresh;
use crate::db::db::*;
use crate::db::db::kline::create_klines_table;
use crate::db::db::historical_squeeze::create_squeeze_table;
use crate::profit_calculator::profit_calculator::run;


fn main() -> () {
    let mut b = bootstrap::Bootstrap::new();
    b.boot();

    info!("Starting up");

    // wow this is ugly
    let wallet_boot = b.clone();
    let pairs_len = b.clone().config.pairs.len();
    let b_copy = b.clone();
    let bootstrap_arc = Arc::new(Mutex::new(b));
    let mut handles = vec![];

    // Wallet synchronization
    thread::spawn(|| refresh(wallet_boot));

    // Profit calculator
    thread::spawn(move || run(b_copy.clone().config));

    // Spawn separate threads with websockets
    for i in 0..pairs_len {
        let bootstrap_arc_clone = Arc::clone(&bootstrap_arc);

        let handle = thread::spawn(move || {
            let bootstrap_lock = bootstrap_arc_clone.lock().unwrap();
            let bootstrap_instance = &bootstrap_lock.clone();
            // uglyyyy
            let symbol = &bootstrap_lock.clone().config.pairs[i].symbol;

            let kline_conn = Connection::open(format!("{}.db", symbol)).unwrap();
            let wallet_conn = Connection::open("wallet.db").unwrap();
            let trade_conn = Connection::open(format!("trades-{}.db", symbol)).unwrap();

            create_klines_table(&kline_conn);
            create_squeeze_table(&trade_conn);

            info!("Opening stream for {}", symbol);
            drop(bootstrap_lock);
            open_kline_stream(&bootstrap_instance, symbol.to_owned(), kline_conn, wallet_conn, trade_conn);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
}
