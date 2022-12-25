use std::ops::Deref;

use serde::{de, Deserialize, Deserializer, Serialize};

use crate::ToChannel;

#[derive(Serialize, Debug)]
pub struct OrderBookQuery {
    pub symbol: String,
    pub limit: u32,
}

pub struct OrderBookChannel {
    pub ticker: String,
}

impl ToChannel for OrderBookChannel {
    fn to_channel(&self) -> String {
        format!("{}@depth", self.ticker.to_lowercase())
    }
}

#[derive(Deserialize)]
pub struct PriceNode(#[serde(deserialize_with = "deser_floats_array_from_string_array")] [f64; 2]);

impl Deref for PriceNode {
    type Target = [f64; 2];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<PriceNode> for [f64; 2] {
    fn from(pn: PriceNode) -> Self {
        pn.0
    }
}

#[derive(Deserialize)]
pub struct WsOrderBook {
    #[serde(rename = "u")]
    pub last_update_id: u64,
    #[serde(rename = "U")]
    pub first_update_id: u64,
    #[serde(rename = "b")]
    pub bids: Vec<PriceNode>,
    #[serde(rename = "a")]
    pub asks: Vec<PriceNode>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiOrderBook {
    pub last_update_id: u64,
    pub bids: Vec<PriceNode>,
    pub asks: Vec<PriceNode>,
}

pub fn deser_floats_array_from_string_array<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<[f64; 2], D::Error> {
    let str_arr = <[String; 2]>::deserialize(deserializer)?;
    let price = str_arr[0].parse().map_err(de::Error::custom)?;
    let quant = str_arr[1].parse().map_err(de::Error::custom)?;
    Ok([price, quant])
}
