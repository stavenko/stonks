use std::time::Duration;

use toolset::{deser_duration_from_integer, deser_float_from_string};
use serde::{Deserialize, Serialize};
use sources_common::time_unit::{ser_time_unit, TimeUnit};

use crate::ToChannel;

pub struct CandleStream {
    pub ticker: String,
    pub time_unit: TimeUnit,
}

impl ToChannel for CandleStream {
    fn to_channel(&self) -> String {
        format!(
            "{}@kline_{}",
            self.ticker.to_lowercase(),
            self.time_unit.fmt()
        )
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CandlesQuery {
    pub symbol: String,
    #[serde(serialize_with = "ser_time_unit")]
    pub interval: TimeUnit,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct Candle {
    #[serde(deserialize_with = "deser_duration_from_integer")]
    pub ts: Duration,
    #[serde(deserialize_with = "deser_float_from_string")]
    pub open: f64,
    #[serde(deserialize_with = "deser_float_from_string")]
    pub high: f64,
    #[serde(deserialize_with = "deser_float_from_string")]
    pub low: f64,
    #[serde(deserialize_with = "deser_float_from_string")]
    pub close: f64,
    #[serde(deserialize_with = "deser_float_from_string")]
    pub volume: f64,
    #[serde(deserialize_with = "deser_duration_from_integer")]
    pub close_ts: Duration,
    #[serde(deserialize_with = "deser_float_from_string")]
    pub quote_volume: f64,
    pub number_of_trades: u32,
    #[serde(deserialize_with = "deser_float_from_string")]
    pub taker_buy_base_asset_volume: f64,
    #[serde(deserialize_with = "deser_float_from_string")]
    pub taker_buy_quote_asset_volume: f64,

    #[serde(rename = "unused")]
    _unused: String,
}

#[derive(Deserialize, Debug)]
pub struct WsCandleData {
    #[serde(rename = "t", deserialize_with = "deser_duration_from_integer")]
    pub ts: Duration,
    #[serde(rename = "T", deserialize_with = "deser_duration_from_integer")]
    pub close_ts: Duration,
    #[serde(rename = "s")]
    pub ticker: String,
    #[serde(rename = "i")]
    pub time_unit: TimeUnit,

    #[serde(rename = "f")]
    pub first_trade_id: u64,
    #[serde(rename = "L")]
    pub last_trade_id: u64,

    #[serde(rename = "o", deserialize_with = "deser_float_from_string")]
    pub open: f64,
    #[serde(rename = "h", deserialize_with = "deser_float_from_string")]
    pub high: f64,
    #[serde(rename = "l", deserialize_with = "deser_float_from_string")]
    pub low: f64,
    #[serde(rename = "c", deserialize_with = "deser_float_from_string")]
    pub close: f64,

    #[serde(rename = "v", deserialize_with = "deser_float_from_string")]
    pub volume: f64,
    #[serde(rename = "n")]
    pub number_of_trades: u32,
    #[serde(rename = "x")]
    pub is_closed: bool,
    #[serde(rename = "q", deserialize_with = "deser_float_from_string")]
    pub quote_volume: f64,

    #[serde(rename = "V", deserialize_with = "deser_float_from_string")]
    pub taker_buy_base_asset_volume: f64,
    #[serde(rename = "Q", deserialize_with = "deser_float_from_string")]
    pub taker_buy_quote_asset_volume: f64,
}

#[derive(Deserialize, Debug)]
pub struct WsCandle {
    #[serde(rename = "k")]
    pub data: WsCandleData,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parsing_candle() {
        let input = r#"
        [
  [
    1499040000000,    
    "0.01634790",     
    "0.80000000",     
    "0.01575800",     
    "0.01577100",     
    "148976.11427815",
    1499644799999,    
    "2434.19055334",  
    308,              
    "1756.87402397",  
    "28.46694368",
    "0"
  ]
]
        "#;
        let candle = serde_json::from_str::<Vec<Candle>>(input).unwrap();

        assert_eq!(candle[0].open, 0.01634790);
        assert_eq!(candle[0].ts.as_secs(), 1499040000);
    }
}
