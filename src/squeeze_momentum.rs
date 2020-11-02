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

    pub fn calculate(klines: Vec<Kline>) {
        let length = 20;
        let mult = 2.0;
        let lengthKC = 20;
        let multKC = 1.5;
        let useTrueRange = true;

        let current_kline = &klines[0];

        let source = &current_kline.close;
        let basis = sma(length, &klines
            .into_iter()
            .take(length as usize)
            .map(|f| {
                f.close.parse::<f64>().unwrap()
            })
            .collect::<Vec<f64>>()
        );
        // let dev = multKC * stdev(length, source);

        info!("SMA: {:?}", basis);

        // Calculate BB
        // source = current_kline.close
    }
}
