pub mod lowest {
    use log::{info, trace, warn};

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
        info!("lowest: {:?}", lowest);
        lowest
    }
}
