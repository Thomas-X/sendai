pub mod util {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn get_now() -> f64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64()
    }
}
