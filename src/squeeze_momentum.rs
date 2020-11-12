pub mod squeeze_momentum {
    extern crate ta_lib_wrapper;

    use binance::model::Kline;
    use log::{info, trace, warn};
    use crate::indicators::indicators::{linreg, sma, lowest, highest, avg_2, trange, stdev};

    static mut last_value: f64 = 0.0;

    pub fn slice_and_map(klines: &Vec<Kline>, len: i32, mapper: fn(&Kline) -> f64) -> Vec<f64> {
        klines
            .into_iter()
            .take(len as usize)
            .map(mapper)
            .collect::<Vec<f64>>()
    }

    pub fn default_mapper(f: &Kline) -> f64 {
        f.close.parse::<f64>().unwrap()
    }

    pub fn calculate(klines: &Vec<Kline>) -> (f64, f64) {
        let length: i32 = 40;
        let mult = 2.0;
        let length_kc: i32 = 40;
        let mult_kc = 1.5;
        let use_true_range = false;
        let low = &slice_and_map(&klines, length, |f| {
            f.low.parse::<f64>().unwrap()
        });
        let high = &slice_and_map(&klines, length, |f| {
            f.high.parse::<f64>().unwrap()
        });
        let close = &slice_and_map(&klines, length, |f| {
            f.close.parse::<f64>().unwrap()
        });

        if length > klines.len() as i32 || length_kc > klines.len() as i32 {
            panic!("length or length_kc was bigger than supplied klines, can't continue")
        }

        // Calculate BB (bollinger bands)
        let basis = sma(length, close);
        let dev = mult_kc * stdev(length, close, 1.0).0[0];
        let upper_bb = basis + dev;
        let lower_bb = basis - dev;

        // Calculate KC
        let ma = sma(length_kc, close);
        let range: Vec<f64> = match use_true_range {
            true => trange(high, low, close).0,
            false => high
                .into_iter()
                .enumerate()
                .map(|(idx, f)| f - &low[idx])
                .collect::<Vec<f64>>()
        };
        let range_ma = sma(length_kc, &range);
        let upper_kc = ma + range_ma * mult_kc;
        let lower_kc = ma - range_ma * mult_kc;

        let squeeze_on = (lower_bb > lower_kc) && (upper_bb < upper_kc);
        let squeeze_off = (lower_bb < lower_kc) && (upper_bb > upper_kc);
        let no_squeeze = (squeeze_on == false) && (squeeze_off == false);

        let super_avg = &avg_2(
            &avg_2(
                &highest(
                    high,
                    length_kc,
                ),
                &lowest(
                    low,
                    length_kc,
                ),
            ),
            &sma(length_kc, close),
        );

        let val = linreg(
            &close
                .into_iter()
                .map(|f| {
                    super_avg - f
                })
                .collect::<Vec<f64>>(),
            length_kc,
        );
        let l_val = unsafe { last_value }.clone();
        unsafe {
            last_value = val.0[0];
        };
        (l_val, val.0[0])
    }
}
