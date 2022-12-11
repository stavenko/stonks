use std::borrow::Cow;

use serde::{de, Deserialize, Deserializer};
use sources_common::time_unit::TimeUnit;
use url::Url;

#[derive(Deserialize)]
pub struct PriceFeedConfig {
    #[serde(deserialize_with = "deserialize_url")]
    pub(super) api_host: Url,
    #[serde(deserialize_with = "deserialize_url")]
    pub(super) ws_host: Url,

    pub(super) ticker: String,
    pub(super) time_unit: TimeUnit,
    #[serde(default = "default_orderbook_depth")]
    pub(super) orderbook_depth: u32,
    #[serde(default = "default_candles_amount")]
    pub(super) candles_amount: usize,
}

fn default_orderbook_depth() -> u32 {
    5000
}

fn default_candles_amount() -> usize {
    50
}

fn deserialize_url<'de, D: Deserializer<'de>>(deser: D) -> Result<Url, D::Error> {
    let s = Cow::<str>::deserialize(deser)?;
    s.as_ref().parse().map_err(de::Error::custom)
}
