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
    LimitMaker,
    Market,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    pub symbol: String,
    pub status: String,
    pub base_asset: String,
    pub base_asset_precision: u16,
    pub quote_asset: String,
    pub quote_precision: u16,
    pub quote_asset_precision: u16,
    pub order_types: Vec<OrderType>,
    pub iceberg_allowed: bool,
    pub oco_allowed: bool,
    pub quote_order_qty_market_allowed: bool,
    pub allow_trailing_stop: bool,
    pub cancel_replace_allowed: bool,
    pub is_spot_trading_allowed: bool,
    pub is_margin_trading_allowed: bool,
    pub permissions: Vec<String>,
    pub default_self_trade_prevention_mode: Option<String>,
    #[serde(default)]
    pub allowed_self_trade_prevention_modes: Vec<String>,
}
