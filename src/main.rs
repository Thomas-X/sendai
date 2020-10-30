mod config;

extern crate binance;

use binance::websockets::*;
use std::sync::atomic::{AtomicBool};
use log::{info, trace, warn};
use simplelog::{TermLogger, LevelFilter, Config, TerminalMode, ConfigBuilder};

#[tokio::main]
async fn main() {
    let logger_config = ConfigBuilder::new()
        .set_time_format_str("%H:%M:%S:%6f")
        .build();
    TermLogger::init(LevelFilter::Info, logger_config, TerminalMode::Mixed);
    info!("Starting up");
    info!("Reading config");

    let config = match config::config() {
        Err(e) => warn!("Could not read config file {}", e),
        Ok(()) => info!("Successfully read config file")
    };

    // let keep_running = AtomicBool::new(true); // Used to control the event loop
    // let kline: String = format!("{}", "ethbtc@kline_1m");
    // let mut web_socket: WebSockets = WebSockets::new(|event: WebsocketEvent| {
    //     match event {
    //         WebsocketEvent::Kline(kline_event) => {
    //             kline_event.symbol
    //         },
    //         _ => (),
    //     };
    //     Ok(())
    // });
    //
    // web_socket.connect(&kline).unwrap(); // check error
    // if let Err(e) = web_socket.event_loop(&keep_running) {
    //     match e {
    //         err => {
    //             println!("Error: {}", err);
    //         }
    //     }
    // }
    // web_socket.disconnect().unwrap();
}
