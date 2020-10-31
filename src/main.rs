mod config;
mod logging;
mod open_kline_stream;
mod handle_kline_event;

use std::sync::atomic::{AtomicBool};
use log::{info, trace, warn};
use std::sync::{Mutex, Arc};
use std::rc::Rc;
use std::thread;

#[tokio::main]
async fn main() {
    logging::logging();
    info!("Starting up");
    let config = match config::config() {
        Err(error) => {
            warn!("Could not read config file, {}", error);
            return ()
        },
        Ok(c) => {
            info!(target: "config", "Successfully read config file");
            c
        }
    };
    let pairs_len = config.pairs.len();
    let pairs = Arc::new(Mutex::new(config.pairs));
    let mut handles = vec![];
    for i in 0..pairs_len {
        let pairs = Arc::clone(&pairs);
        let handle = thread::spawn(move || {
            let mut pairs_lock = pairs.lock().unwrap();
            let symbol = pairs_lock[i].symbol.clone();
            info!("Opening stream for {}", symbol);
            // we can't rely on Rust's out-of-scope dropping of lock because we're doing a sync call that never makes the lock go out of scope.
            // we know what we're doing though, the compiler doesn't
            drop(pairs_lock);
            open_kline_stream::open_kline_stream(handle_kline_event::handle_kline_event, symbol);
        });
        handles.push(handle)
    }
    for handle in handles {
        handle.join().unwrap();
    }
}
