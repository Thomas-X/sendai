use binance::model::KlineEvent;
use log::{info, trace, warn};

pub(crate) fn handle_kline_event(kline_event: KlineEvent) {
    info!("{}: {}", kline_event.symbol, kline_event.kline.volume)
}
