pub mod sma {
    pub fn sum(prices: &Vec<f64>) -> f64 {
        let mut sum: f64 = 0.0;
        for price in prices {
            sum += price;
        }
        sum
    }

    pub fn sma(period: i32, close_prices: &Vec<f64>) -> f64 {
        sum(close_prices) / period as f64
    }
}

