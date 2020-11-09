pub mod indicators {
    extern crate ta_lib_wrapper;

    use log::{info, trace, warn};
    use ta_lib_wrapper::{TA_Integer, TA_Real, TA_LINEARREG, TA_STDDEV, TA_TRANGE, TA_RetCode};

    pub fn avg_2(one: &f64, two: &f64) -> f64 {
        (one + two) / 2 as f64
    }

    pub fn highest(series: &Vec<f64>, period: i32) -> f64 {
        let mut highest: f64 = series[0];
        series
            .into_iter()
            .take(period as usize)
            .map(|f| {
                if f >= &highest {
                    highest = *f;
                }
                f
            })
            .for_each(drop);
        highest
    }

    pub fn linreg(series: &Vec<TA_Real>, period: i32) -> (Vec<TA_Real>, TA_Integer) {
        let mut out: Vec<TA_Real> = Vec::with_capacity(series.len());
        let mut out_begin: TA_Integer = 0;
        let mut out_size: TA_Integer = 0;

        unsafe {
            let ret_code = TA_LINEARREG(
                // all the high/low/close are the same length, so we just grab the first
                0,                              // index of the first close to use
                series.len() as i32 - 1,  // index of the last close to use
                series.as_ptr(),
                period,
                &mut out_begin,                 // set to index of the first close to have an rsi value
                &mut out_size,                  // set to number of sma values computed
                out.as_mut_ptr(),                // pointer to the first element of the output vector
            );
            match ret_code {
                // Indicator was computed correctly, since the vector was filled by TA-lib C library,
                // Rust doesn't know what is the new length of the vector, so we set it manually
                // to the number of values returned by the TA_RSI call
                TA_RetCode::TA_SUCCESS => out.set_len(out_size as usize),
                // An error occured
                _ => panic!("Could not compute indicator, err: {:?}", ret_code)
            }
        }

        (out, out_begin)
    }

    pub fn stdev(period: i32, close_prices: &Vec<TA_Real>, nbdev: f64) -> (Vec<TA_Real>, TA_Integer) {
        let mut out: Vec<TA_Real> = Vec::with_capacity(close_prices.len());
        let mut out_begin: TA_Integer = 0;
        let mut out_size: TA_Integer = 0;

        unsafe {
            let ret_code = TA_STDDEV(
                0,                              // index of the first close to use
                close_prices.len() as i32 - 1,  // index of the last close to use
                close_prices.as_ptr(),          // pointer to the first element of the vector
                period as i32,                  // period of the stddev
                nbdev as f64,
                &mut out_begin,                 // set to index of the first close to have an rsi value
                &mut out_size,                  // set to number of sma values computed
                out.as_mut_ptr(),                // pointer to the first element of the output vector
            );
            match ret_code {
                // Indicator was computed correctly, since the vector was filled by TA-lib C library,
                // Rust doesn't know what is the new length of the vector, so we set it manually
                // to the number of values returned by the TA_RSI call
                TA_RetCode::TA_SUCCESS => out.set_len(out_size as usize),
                // An error occured
                _ => panic!("Could not compute indicator, err: {:?}", ret_code)
            }
        }

        (out, out_begin)
    }

    pub fn trange(high: &Vec<TA_Real>, low: &Vec<TA_Real>, close: &Vec<TA_Real>) -> (Vec<TA_Real>, TA_Integer) {
        let mut out: Vec<TA_Real> = Vec::with_capacity(high.len());
        let mut out_begin: TA_Integer = 0;
        let mut out_size: TA_Integer = 0;

        unsafe {
            let ret_code = TA_TRANGE(
                // all the high/low/close are the same length, so we just grab the first
                0,                              // index of the first close to use
                high.len() as i32 - 1,  // index of the last close to use
                high.as_ptr(),
                low.as_ptr(),
                close.as_ptr(),
                &mut out_begin,                 // set to index of the first close to have an rsi value
                &mut out_size,                  // set to number of sma values computed
                out.as_mut_ptr(),                // pointer to the first element of the output vector
            );
            match ret_code {
                // Indicator was computed correctly, since the vector was filled by TA-lib C library,
                // Rust doesn't know what is the new length of the vector, so we set it manually
                // to the number of values returned by the TA_RSI call
                TA_RetCode::TA_SUCCESS => out.set_len(out_size as usize),
                // An error occured
                _ => panic!("Could not compute indicator, err: {:?}", ret_code)
            }
        }

        (out, out_begin)
    }

    pub fn lowest(series: &Vec<f64>, period: i32) -> f64 {
        let mut lowest: f64 = series[0];
        series
            .into_iter()
            .take(period as usize)
            .map(|f| {
                if f <= &lowest {
                    lowest = *f;
                }
                f
            })
            .for_each(drop);
        lowest
    }

    pub fn sum(prices: &Vec<f64>) -> f64 {
        let mut sum: f64 = 0.0;
        for price in prices {
            sum += price;
        }
        sum
    }

    pub fn sma(period: i32, close: &Vec<f64>) -> f64 {
        sum(close) / period as f64
    }
}
