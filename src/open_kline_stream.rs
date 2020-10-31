extern crate binance;

use binance::websockets::*;
use std::sync::atomic::AtomicBool;
use self::binance::model::KlineEvent;
use log::{info, trace, warn};

type KlineHandler = fn(KlineEvent) -> ();

pub(crate) fn open_kline_stream(handler: KlineHandler, symbol: String) {
    let keep_running = AtomicBool::new(true);
    let kline: String = format!("{}", format!("{}@kline_1m", symbol.to_lowercase()));
    let mut web_socket: WebSockets = WebSockets::new(|event: WebsocketEvent| {
        match event {
            WebsocketEvent::Kline(kline_event) => handler(kline_event),
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
