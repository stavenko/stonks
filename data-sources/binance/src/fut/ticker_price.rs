use std::time::Duration;

use toolset::{deser_duration_from_integer, deser_float_from_string};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SymbolPrice {
    #[serde(deserialize_with = "deser_float_from_string")]
    pub price: f64,
    #[serde(deserialize_with = "deser_duration_from_integer")]
    pub time: Duration,
    pub symbol: String,
}
