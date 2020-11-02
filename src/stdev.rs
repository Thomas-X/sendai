pub mod stdev {
    // use crate::sma::sma::{sma, sum};
    //
    // pub fn stdev(period: u32, close_prices: &Vec<TA_Real>) -> (Vec<TA_Real>, TA_Integer) {
    //     let mut out: Vec<TA_Real> = Vec::with_capacity(close_prices.len());
    //     let mut out_begin: TA_Integer = 0;
    //     let mut out_size: TA_Integer = 0;
    //
    //     unsafe {
    //         let ret_code = TA_RSI(
    //             0,                              // index of the first close to use
    //             close_prices.len() as i32 - 1,  // index of the last close to use
    //             close_prices.as_ptr(),          // pointer to the first element of the vector
    //             period as i32,                  // period of the rsi
    //             &mut out_begin,                 // set to index of the first close to have an rsi value
    //             &mut out_size,                  // set to number of sma values computed
    //             out.as_mut_ptr()                // pointer to the first element of the output vector
    //         );
    //         match ret_code {
    //             // Indicator was computed correctly, since the vector was filled by TA-lib C library,
    //             // Rust doesn't know what is the new length of the vector, so we set it manually
    //             // to the number of values returned by the TA_RSI call
    //             TA_RetCode::TA_SUCCESS => out.set_len(out_size as usize),
    //             // An error occured
    //             _ => panic!("Could not compute indicator, err: {:?}", ret_code)
    //         }
    //     }
    //
    //     (out, out_begin)
    // }
}

