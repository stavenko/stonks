use std::time::Duration;

use binance::{candle::CandleStream, orderbook::OrderBookChannel, ToChannel};
use candle::Candle;
use order_book::OrderBook;
use sources_common::time_unit::TimeUnit;
use url::Url;

pub mod trade;
pub mod candle;
pub mod candles;
pub mod order_book;

#[derive(Debug)]
pub enum MarketFeedMessage {
    Candle(Candle),
    OrderBook(OrderBook),
}

pub enum MarketFeedSettings {
    Candle(TimeUnit),
    OrderBook,
    Trades,
}

pub struct MarketFeedInput {
    pub ticker: String,
    pub ws_url: Url,
    pub settings: Vec<MarketFeedSettings>,
}


pub struct FetchCandlesInput {
    pub api_host: Url,
    pub ticker: String,
    pub time_unit: TimeUnit,
    pub countback: usize,
}

pub struct FetchOrderbookInput {
    pub api_host: Url,
    pub ticker: String,
    pub depth: u32,
}

pub struct FetchHistoricalTradesInput {
    pub api_host: Url,
    pub api_key: String,
    pub ticker: String,
    pub from: Duration,
}

#[cfg(feature = "binance")]
mod binance_adaptor;

#[cfg(feature = "binance")]
pub use binance_adaptor::*;


