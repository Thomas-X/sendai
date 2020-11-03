pub mod highest {
    use log::{info, trace, warn};

    pub fn highest (series: &Vec<f64>, period: i32) -> f64 {
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
}
