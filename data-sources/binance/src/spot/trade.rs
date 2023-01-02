use std::time::Duration;

use serde::Deserialize;
use toolset::{deser_duration_from_integer, deser_float_from_string};

#[derive(Deserialize)]
pub struct WsTrade{
    #[serde(rename="t")]
    pub id: u64,
    #[serde(rename="s")]
    pub symbol: String,
    #[serde(rename="p", deserialize_with = "deser_float_from_string")]
    pub price: f64,
    #[serde(rename="q", deserialize_with = "deser_float_from_string")]
    pub qty: f64,
    #[serde(rename="T", deserialize_with = "deser_duration_from_integer")]
    pub time: Duration, // Trade executed timestamp, as same as `T` in the stream

    #[serde(rename="m")]
    pub is_buyer_maker: bool,
    #[serde(rename="M")]
    pub is_best_match: bool,
}

impl WsTrade {
    pub fn quote_qty(&self) -> f64 {
        self.price * self.qty
    }
}

