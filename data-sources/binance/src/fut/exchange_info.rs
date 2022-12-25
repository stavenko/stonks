use std::time::Duration;

use toolset::deser_duration_from_integer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Default, Serialize)]
pub struct ExchangeInfoRequest {
    pub symbol: Option<String>,
    pub symbols: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfo {
    pub timezone: String,
    #[serde(deserialize_with = "deser_duration_from_integer")]
    pub server_time: Duration,
    pub symbols: Vec<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Limit,
    Market,
    Stop,
    StopMarket,
    TakeProfit,
    TakeProfitMarket,
    TrailingStopMarket,
}

impl Default for OrderType {
    fn default() -> Self {
        Self::TakeProfitMarket
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    pub symbol: String,
    pub status: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub order_types: Vec<OrderType>,
    pub contract_type: String,
}
