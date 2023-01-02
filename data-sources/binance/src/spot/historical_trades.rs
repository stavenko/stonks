use crate::ToChannel;
use std::time::Duration;
use toolset::{deser_duration_from_integer, deser_float_from_string};

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct HistoricalTradesQuery {
    pub query: Query,
    pub api_key: String,
}

#[derive(Debug)]
pub struct AllHistoricalTradesQuery {
    pub query: AllQuery,
    pub api_key: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Query {
    pub symbol: String,
    pub from_id: Option<u64>,
}

#[derive(Debug)]
pub struct AllQuery {
    pub ticker: String,
    pub window: Duration,
}

pub struct HistoricalTradesChannel {
    pub ticker: String,
}

impl ToChannel for HistoricalTradesChannel {
    fn to_channel(&self) -> String {
        format!("{}@trade", self.ticker.to_lowercase())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiHistoricalTrade {
    pub id: u64,
    #[serde(deserialize_with = "deser_float_from_string")]
    pub price: f64,
    #[serde(deserialize_with = "deser_float_from_string")]
    pub qty: f64,
    #[serde(deserialize_with = "deser_float_from_string")]
    pub quote_qty: f64,
    #[serde(deserialize_with = "deser_duration_from_integer")]
    pub time: Duration, // Trade executed timestamp, as same as `T` in the stream
    pub is_buyer_maker: bool,
    pub is_best_match: bool,
}

