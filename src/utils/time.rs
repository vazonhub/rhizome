use std::time::{SystemTime, UNIX_EPOCH};

/// Return current time in seconds in i64 format
pub fn get_now_i64() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
/// Return current time in seconds in f64 format
pub fn get_now_f64() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
