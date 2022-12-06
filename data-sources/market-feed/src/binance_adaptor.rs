use binance::{
    candle::{CandleStream, WsCandle},
    orderbook::{self, OrderBookChannel, OrderBookQuery},
    protocol::{StreamData, StreamPackage},
};
use futures::{Stream, StreamExt};
use sources_common::time_unit::TimeUnit;
use tracing::{debug, error, info};

use crate::{
    candle::Candle, candles::Candles, order_book::OrderBook, FetchCandlesInput,
    FetchOrderbookInput, MarketFeedInput, MarketFeedMessage,
};

pub async fn fetch_candles(input: FetchCandlesInput) -> Candles {
    use binance::candle::CandlesQuery;
    use binance::fetch_candles;
    let bin_candles = fetch_candles(
        input.api_host,
        CandlesQuery {
            symbol: input.ticker,
            interval: input.time_unit.clone(),
            start_time: None,
            end_time: None,
            limit: None,
        },
    )
    .await;

    bin_candles
        .into_iter()
        .map(|c| (c, input.time_unit.clone()))
        .collect::<Vec<_>>()
        .into()
}

pub async fn fetch_orderbook(input: FetchOrderbookInput) -> OrderBook {
    let orderbook = binance::fetch_orderbook(
        input.api_host,
        OrderBookQuery {
            limit: input.depth,
            symbol: input.ticker,
        },
    )
    .await;

    orderbook.into()
}

pub async fn create_market_feed(
    input: MarketFeedInput,
) -> Option<impl Stream<Item = MarketFeedMessage> + Send + Sync> {
    let stream = binance::get_market_stream(
        input.ws_url,
        vec![
            Box::new(CandleStream {
                ticker: input.ticker.clone(),
                time_unit: input.time_unit,
            }),
            Box::new(OrderBookChannel {
                ticker: input.ticker,
            }),
        ],
    )
    .await;

    Some(stream.filter_map(|item| async move {
        match item
            .map_err(|e| format!("Binance error: {e}"))
            .and_then(TryInto::try_into)
        {
            Ok(item) => Some(item),
            Err(e) => {
                info!("non-data message {}", e);
                None
            }
        }
    }))
}

impl From<binance::orderbook::WsOrderBook> for OrderBook {
    fn from(ob: binance::orderbook::WsOrderBook) -> Self {
        Self {
            asks: ob.asks.into_iter().map(Into::into).collect(),
            bids: ob.bids.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<binance::orderbook::ApiOrderBook> for OrderBook {
    fn from(ob: binance::orderbook::ApiOrderBook) -> Self {
        Self {
            asks: ob.asks.into_iter().map(Into::into).collect(),
            bids: ob.bids.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<WsCandle> for Candle {
    fn from(candle: WsCandle) -> Self {
        let candle = candle.data;
        Candle {
            ts: candle.ts,
            time_unit: candle.time_unit,
            open: candle.open,
            high: candle.high,
            low: candle.low,
            close: candle.close,
            volume: candle.volume,
            quote_volume: candle.quote_volume,
        }
    }
}

impl From<StreamPackage> for MarketFeedMessage {
    fn from(package: StreamPackage) -> Self {
        debug!(
            ?package,
            "Transforming stream package to market feed message"
        );
        match package.event.event_type.as_ref() {
            "kline" => {
                let value = package.event.data();
                match serde_json::from_value::<binance::candle::WsCandle>(value.clone()) {
                    Ok(candle) => MarketFeedMessage::Candle(candle.into()),
                    Err(e) => {
                        error!("Parse error {:?} <{e}>", value);
                        panic!();
                    }
                }
            }
            "depthUpdate" => {
                let ob: orderbook::WsOrderBook =
                    serde_json::from_value(package.event.data()).unwrap();
                MarketFeedMessage::OrderBook(ob.into())
            }
            _ => panic!("package stream: {}", package.stream),
        }
    }
}

impl From<(binance::candle::Candle, TimeUnit)> for Candle {
    fn from((c, tu): (binance::candle::Candle, TimeUnit)) -> Self {
        Self {
            time_unit: tu,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            ts: c.ts,
            volume: c.volume,
            quote_volume: c.quote_volume,
        }
    }
}

impl From<Vec<(binance::candle::Candle, TimeUnit)>> for Candles {
    fn from(cs: Vec<(binance::candle::Candle, TimeUnit)>) -> Self {
        Candles::new(cs.into_iter().map(Into::into).collect())
    }
}

impl TryFrom<StreamData> for MarketFeedMessage {
    type Error = String;
    fn try_from(value: StreamData) -> Result<Self, Self::Error> {
        match value {
            StreamData::Package(p) => Ok(p.into()),
            StreamData::SubscribeResponse { response, id } => Err(format!(
                "wtf: {} ({id})",
                serde_json::to_string_pretty(&response).unwrap()
            )),
        }
    }
}
