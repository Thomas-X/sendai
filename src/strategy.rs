pub mod strategy {
    use log::{info, trace, warn};
    use binance::model::Kline;
    use crate::squeeze_momentum::squeeze_momentum;
    use rusqlite::{params, Connection, Result, Statement};
    use std::time::{UNIX_EPOCH, SystemTime};
    use crate::db::db::trade::{Trade};
    use crate::db::db::historical_squeeze::{create_squeeze, Squeeze, get_squeeze_value};
    use crate::util::util::get_now;
    use std::borrow::Borrow;
    use crate::bootstrap::Config;
    // [CHECK] TODO: implement "wallet" balance mechanism
    // [CHECK] TODO: implement trade model, but could also just be a create_table if not exists
    // TODO: implement "buying" mechanism
    // TODO: this should be a simple market order, with fees taken into account
    // TODO: implement selling mechanism
    // TODO: market order too

    // TODO: if everything is functional, add a continous-improving of parameters mode

    // todo add proper last value handling, atm it's just the last calculation but a calculation happens multiple times
    // todo per bar, so it should really just be the final calculation of the final bar

    pub fn get_quarantine_bars(trade_conn: &Connection, config: &Config) -> Vec<Trade> {
        let quarantine_time: i64 = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - (60 * config.quarantine_interval_in_min) as u64) as i64;
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
        info!("-----------------------------------");
        info!("Start calculation");
        let (last_value, current_value) = squeeze_momentum::calculate(&klines);
        create_squeeze(trade_conn, &Squeeze {
            value: current_value,
            price_in_stake: klines.first().unwrap().close.parse::<f64>().unwrap(),
            timestamp: get_now(),
        });
        // 2 = every 2 seconds we get 1 new squeeze value
        // 300 = amount of squeeze values we need for startup
        // 43200 = 24h of continious kline ticking
        let amount_of_squeezes_needed = 43200;
        let startup_time: f64 = (2 * amount_of_squeezes_needed) as f64;
        let (squeeze_avg_negative, negative_squeeze_count) = get_squeeze_value(&trade_conn, get_now() - &startup_time, true);
        let (squeeze_avg_positive, positive_squeeze_count) = get_squeeze_value(&trade_conn, get_now() - &startup_time, false);
        info!("negative_squeeze_count, squeeze_avg_negative: {:?} {:?}", negative_squeeze_count, squeeze_avg_negative);
        info!("positive_squeeze_count, squeeze_avg_positive: {:?} {:?}", positive_squeeze_count, squeeze_avg_positive);
        info!("Current squeeze value: {:?}", current_value);
        // 1 hour mimimum warmup time and if we're currently in a negative setup
        if negative_squeeze_count >= 300 && current_value < 0.0 {
            info!("-----------------------------------");
            info!("End calculation");
            return (current_value <= squeeze_avg_negative && current_value <= last_value, false);
        } else if negative_squeeze_count < 300 {
            info!("Not buying because squeeze amount is too low for avg {:?}", negative_squeeze_count);
        }
        if positive_squeeze_count >= 300 && current_value > 0.0 {
            // only buy if we're above the avg
            info!("End calculation");
            info!("-----------------------------------");
            return (false, current_value >= squeeze_avg_positive)
        } else if positive_squeeze_count < 300 {
            info!("Not selling because squeeze amount is too low for avg {:?}", positive_squeeze_count);
        }
        info!("End calculation");
        info!("-----------------------------------");
        return (false, false);
    }
}
