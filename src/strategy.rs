pub mod strategy {
    use log::{info, trace, warn};
    use binance::model::Kline;

    pub fn should_buy(klines: Vec<Kline>) {
        info!("buy called")
    }

    pub fn should_sell() {
        info!("sell called")
    }
}
