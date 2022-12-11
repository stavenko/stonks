use std::time::Duration;

use sources_common::time_unit::TimeUnit;

#[derive(Debug, Clone, PartialEq)]
pub struct Candle {
    pub ts: Duration,
    pub time_unit: TimeUnit,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub quote_volume: f64,
}

impl Eq for Candle {
}
