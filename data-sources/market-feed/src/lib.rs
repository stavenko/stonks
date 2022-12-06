use candle::Candle;
use order_book::OrderBook;
use sources_common::time_unit::TimeUnit;
use url::Url;

pub mod candle;
pub mod candles;
pub mod order_book;

#[derive(Debug)]
pub enum MarketFeedMessage {
    Candle(Candle),
    OrderBook(OrderBook),
}

pub struct MarketFeedInput {
    pub ticker: String,
    pub time_unit: TimeUnit,
    pub ws_url: Url,
}

pub struct FetchCandlesInput {
    pub api_host: Url,
    pub ticker: String,
    pub time_unit: TimeUnit,
}

pub struct FetchOrderbookInput {
    pub api_host: Url,
    pub ticker: String,
    pub depth: u32,
}

#[cfg(feature = "binance")]
mod binance_adaptor;

#[cfg(feature = "binance")]
pub use binance_adaptor::*;


