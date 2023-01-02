use std::{borrow::Cow, time::Duration};

use serde::{de, Deserialize, Deserializer};
use sources_common::time_unit::{TimeUnit, DAY};
use url::Url;

use super::AggregateOptions;

#[derive(Deserialize, Debug)]
pub struct CandleSettings {
    pub(super) time_unit: TimeUnit,
    #[serde(default = "default_candles_amount")]
    pub(super) amount: usize,
}

#[derive(Deserialize, Debug)]
pub struct OrderbookSettings {
    #[serde(default = "default_orderbook_depth")]
    pub(super) depth: u32,
}

#[derive(Deserialize, Debug)]
pub struct TradesSettings {
    #[serde(default = "default_trades_duration", with = "humantime_serde")]
    pub(super) window: Duration,
}

#[derive(Deserialize)]
pub struct PriceFeedConfig {
    #[serde(deserialize_with = "deserialize_url")]
    pub(super) api_host: Url,
    #[serde(deserialize_with = "deserialize_url")]
    pub(super) ws_host: Url,

    #[serde(default = "api_key_from_env")]
    pub(super) api_key: String,

    pub(super) ticker: String,
    pub(super) candles: Option<CandleSettings>,
    pub(super) orderbook: Option<OrderbookSettings>,
    pub(super) trades: Option<TradesSettings>,
    pub(super) aggregate_options: AggregateOptions,
}

fn api_key_from_env() -> String {
    std::env::var("BINANCE_API_KEY")
        .expect("Provide api key via config or via env var 'BINANCE_API_KEY'")
}

fn default_orderbook_depth() -> u32 {
    5000
}

fn default_candles_amount() -> usize {
    50
}

fn default_trades_duration() -> Duration {
    Duration::from_secs(DAY as u64)
}

fn deserialize_url<'de, D: Deserializer<'de>>(deser: D) -> Result<Url, D::Error> {
    let s = Cow::<str>::deserialize(deser)?;
    s.as_ref().parse().map_err(de::Error::custom)
}
