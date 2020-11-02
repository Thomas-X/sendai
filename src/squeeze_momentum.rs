// the following indicators are needed:
// sma
// stdev
// linear regression curve
// avg of all series
// highest value in series

// Pinescript -> rust
//
// @author LazyBear
// List of all my indicators: https://www.tradingview.com/v/4IneGo8h/
//
// study(shorttitle = "SQZMOM_LB", title="Squeeze Momentum Indicator [LazyBear]", overlay=false)
//
// length = input(20, title="BB Length")
// mult = input(2.0,title="BB MultFactor")
// lengthKC=input(20, title="KC Length")
// multKC = input(1.5, title="KC MultFactor")
//
// useTrueRange = input(true, title="Use TrueRange (KC)", type=bool)
//
// // Calculate BB
// source = close
// basis = sma(source, length)
// dev = multKC * stdev(source, length)
// upperBB = basis + dev
// lowerBB = basis - dev
//
// // Calculate KC
// ma = sma(source, lengthKC)
// range = useTrueRange ? tr : (high - low)
// rangema = sma(range, lengthKC)
// upperKC = ma + rangema * multKC
// lowerKC = ma - rangema * multKC
//
// sqzOn  = (lowerBB > lowerKC) and (upperBB < upperKC)
// sqzOff = (lowerBB < lowerKC) and (upperBB > upperKC)
// noSqz  = (sqzOn == false) and (sqzOff == false)
//
// val = linreg(source  -  avg(avg(highest(high, lengthKC), lowest(low, lengthKC)),sma(close,lengthKC)),
// lengthKC,0)
//
// bcolor = iff( val > 0,
// iff( val > nz(val[1]), lime, green),
// iff( val < nz(val[1]), red, maroon))
// scolor = noSqz ? blue : sqzOn ? black : gray
// plot(val, color=bcolor, style=histogram, linewidth=4)
// plot(0, color=scolor, style=cross, linewidth=2)


pub mod squeeze_momentum {
    extern crate ta_lib_wrapper;

    use binance::model::Kline;
    use log::{info, trace, warn};
    use crate::sma::sma::sma;
    use crate::stdev::stdev::stdev;
    use crate::trange::trange::trange;
    use crate::linreg::linreg::linreg;
    use crate::avg::avg::avg_2;
    use crate::highest::highest::highest;
    use crate::lowest::lowest::lowest;

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

    pub fn calculate(klines: Vec<Kline>) {
        let length: i32 = 20;
        let mult = 2.0;
        let length_kc: i32 = 20;
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
        info!("super_avg {:?}", super_avg);
        info!("close first {:?}", close.first().unwrap());
        info!("close last {:?}", close.last().unwrap());
        info!("super_avg - close {:?}", &close
            .into_iter()
            .map(|f| {
                f - super_avg
            })
            .collect::<Vec<f64>>());
        info!("val: {:?}", val.0[0]);
        // info!("squeeze_on: {:?}", squeeze_on);
        // info!("squeeze_off: {:?}", squeeze_off);
        // info!("no_squeeze: {:?}", no_squeeze);
    }
}
