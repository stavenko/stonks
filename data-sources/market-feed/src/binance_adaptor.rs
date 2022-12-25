use std::time::{SystemTime, UNIX_EPOCH};

use binance::{
    protocol::{StreamData, StreamPackage},
    spot::{
        candle::{CandlesQuery, WsCandle, CandleStream},
        historical_trades::{AllHistoricalTradesQuery, HistoricalTradesChannel},
        orderbook::{OrderBookQuery, OrderBookChannel},
    },
    ToChannel,
};
use futures::{Stream, StreamExt};
use sources_common::time_unit::TimeUnit;
use tracing::{debug, error, info};

use crate::{
    candle::Candle,
    candles::Candles,
    order_book::OrderBook,
    trade::{Trade, Trades},
    FetchCandlesInput, FetchHistoricalTradesInput, FetchOrderbookInput, MarketFeedInput,
    MarketFeedMessage, MarketFeedSettings,
};

pub async fn fetch_candles(input: FetchCandlesInput) -> Candles {
    let from = SystemTime::now() - input.time_unit.calc_n(50);
    let to = SystemTime::now() + input.time_unit.calc_n(1);
    let from = from.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let to = to.duration_since(UNIX_EPOCH).unwrap().as_secs();

    info!(
        "from {} now: {}",
        from,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let bin_candles = binance::spot::fetch_candles(
        input.api_host,
        CandlesQuery {
            symbol: input.ticker,
            interval: input.time_unit.clone(),
            start_time: Some(from * 1000),
            end_time: None,
            limit: None,
            offset: None,
        },
    )
    .await;

    info!("Fetched {} candles", bin_candles.len());

    bin_candles
        .into_iter()
        .map(|c| (c, input.time_unit.clone()))
        .collect::<Vec<_>>()
        .into()
}

pub async fn fetch_orderbook(input: FetchOrderbookInput) -> OrderBook {
    let orderbook = binance::spot::fetch_orderbook(
        input.api_host,
        OrderBookQuery {
            limit: input.depth,
            symbol: input.ticker,
        },
    )
    .await;

    orderbook.into()
}

pub async fn fetch_historical_trades(input: FetchHistoricalTradesInput) -> Trades {
    let trades = binance::spot::fetch_all_historical_trades(
        input.api_host,
        AllHistoricalTradesQuery {
            api_key: input.api_key,
            query: binance::spot::historical_trades::AllQuery {
                ticker: input.ticker,
                from_date: input.from,
            },
        },
    )
    .await;

    Trades::new(trades.into_iter().map(Into::into).collect())
}

pub async fn create_market_feed(
    input: MarketFeedInput,
) -> Option<impl Stream<Item = MarketFeedMessage> + Send + Sync> {
    let channels = input.get_channels();
    let stream = binance::spot::get_market_stream(input.ws_url, channels).await;

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

impl From<binance::spot::historical_trades::ApiHistoricalTrade> for Trade {
    fn from(item: binance::spot::historical_trades::ApiHistoricalTrade) -> Self {
        Self {
            price: item.price,
            quantity: item.qty,
            quote_quantity: item.quote_qty,
            time: item.time,
        }
    }
}
impl From<binance::spot::orderbook::WsOrderBook> for OrderBook {
    fn from(ob: binance::spot::orderbook::WsOrderBook) -> Self {
        Self {
            asks: ob.asks.into_iter().map(Into::into).collect(),
            bids: ob.bids.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<binance::spot::orderbook::ApiOrderBook> for OrderBook {
    fn from(ob: binance::spot::orderbook::ApiOrderBook) -> Self {
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
                match serde_json::from_value::<binance::spot::candle::WsCandle>(value.clone()) {
                    Ok(candle) => MarketFeedMessage::Candle(candle.into()),
                    Err(e) => {
                        error!("Parse error {:?} <{e}>", value);
                        panic!();
                    }
                }
            }
            "depthUpdate" => {
                let ob: binance::spot::orderbook::WsOrderBook =
                    serde_json::from_value(package.event.data()).unwrap();
                MarketFeedMessage::OrderBook(ob.into())
            }
            _ => panic!("package stream: {}", package.stream),
        }
    }
}

impl From<(binance::spot::candle::Candle, TimeUnit)> for Candle {
    fn from((c, tu): (binance::spot::candle::Candle, TimeUnit)) -> Self {
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

impl From<Vec<(binance::spot::candle::Candle, TimeUnit)>> for Candles {
    fn from(cs: Vec<(binance::spot::candle::Candle, TimeUnit)>) -> Self {
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

impl MarketFeedInput {
    fn get_channels(&self) -> Vec<Box<dyn ToChannel + Send>> {
        self.settings
            .iter()
            .map(|settings| {
                let b: Box<dyn ToChannel + Send> = match settings {
                    MarketFeedSettings::Candle(tu) => Box::new(CandleStream {
                        ticker: self.ticker.clone(),
                        time_unit: tu.clone(),
                    }),
                    MarketFeedSettings::OrderBook => Box::new(OrderBookChannel {
                        ticker: self.ticker.clone(),
                    }),
                    MarketFeedSettings::Trades => Box::new(HistoricalTradesChannel {
                        ticker: self.ticker.clone(),
                    }),
                };
                b
            })
            .collect()
    }
}
