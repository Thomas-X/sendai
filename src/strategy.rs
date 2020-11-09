pub mod strategy {
    use log::{info, trace, warn};
    use binance::model::Kline;
    use crate::squeeze_momentum::squeeze_momentum;
    use rusqlite::{params, Connection, Result, Statement};
    use std::time::{UNIX_EPOCH, SystemTime};
    use crate::db::db::trade::{Trade};
    // [CHECK] TODO: implement "wallet" balance mechanism
    // [CHECK] TODO: implement trade model, but could also just be a create_table if not exists
    // TODO: implement "buying" mechanism
    // TODO: this should be a simple market order, with fees taken into account
    // TODO: implement selling mechanism
    // TODO: market order too

    // TODO: if everything is functional, add a continous-improving of parameters mode

    // todo add proper last value handling, atm it's just the last calculation but a calculation happens multiple times
    // todo per bar, so it should really just be the final calculation of the final bar

    pub fn get_quarantine_bars(trade_conn: &Connection) -> Vec<Trade> {
        let quarantine_time: i64 = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - (60 * 20)) as i64;
        let mut stmt = trade_conn.prepare("SELECT * FROM trades WHERE start_bar_time > ?1").unwrap();
        let trades = stmt.query_map(params![quarantine_time], |row| {
            Ok(Trade {
                id: row.get(0).unwrap(),
                amount_crypto: row.get(1).unwrap(),
                amount_money: row.get(2).unwrap(),
                start_bar_time: row.get(3).unwrap(),
            })
        })
            .unwrap()
            .map(|f| f.unwrap())
            .collect();
        trades
    }

    pub fn calculate(klines: &Vec<Kline>, trade_conn: &Connection) -> (bool, bool) {
        let (last_value, current_value) = squeeze_momentum::calculate(&klines);
        info!("Current squeeze value: {:?}", current_value);
        if current_value > 0.0 {
            return if current_value >= 25.9 {
                (current_value > last_value, false)
            } else {
                (false, false)
            }
        } else if current_value < 0.0 {
            // -30 is the lower band of the indicator (see tradingview https://www.tradingview.com/chart/LX2mHohb/)
            let quarantine_trades = get_quarantine_bars(&trade_conn);
            return if current_value <= -26.9 && quarantine_trades.len() < 15 {
                return (false, current_value < last_value)
            } else {
                (false, false)
            }
        }
        return (false, false)
    }
}
