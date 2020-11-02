pub mod strategy {
    use log::{info, trace, warn};
    use binance::model::Kline;
    use crate::squeeze_momentum::squeeze_momentum;

    pub fn should_buy(klines: Vec<Kline>) {
        squeeze_momentum::calculate(klines);
    }

    pub fn should_sell() {
        info!("sell called")
    }
}
