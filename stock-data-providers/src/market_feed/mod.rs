use std::time::Duration;

use app::{mpsc, worker::ProducerWorker, BoxFuture, FutureExt, SinkExt, StreamExt};
use futures::select;
use market_feed::{candles::Candles, order_book::OrderBook, trade::{TradesAggregate, }};
use serde::Deserialize;
use sources_common::symbol::Symbol;
use tracing::info;
use url::Url;

use self::config::{CandleSettings, OrderbookSettings, TradesSettings};

pub mod config;
mod implementation;

#[derive(Default, Debug, Clone)]
pub struct PriceFeedData {
    pub candles: Option<Candles>,
    pub orderbook: Option<OrderBook>,
    pub trades_aggregate: Option<TradesAggregate>,
}

#[derive(Deserialize, Default, Debug, Clone, PartialEq)]
pub struct AggregateOptions {
    tolerance: f64,
    #[serde(with="humantime_serde")]
    speed_factor_window: Duration,
}


#[derive(Debug)]
pub struct PriceFeed {
    api_host: Url,
    api_key: String,
    ws_host: Url,
    ticker: String,
    candles: Option<CandleSettings>,
    orderbook: Option<OrderbookSettings>,
    trades: Option<TradesSettings>,
    aggregate_options: AggregateOptions,
}

impl<'f> ProducerWorker<'f, PriceFeedData> for PriceFeed {
    fn work(self: Box<Self>, mut state_tx: app::mpsc::Sender<PriceFeedData>) -> BoxFuture<'f, ()> {
        async move {
            let mut accumulated = PriceFeedData::default();
            let (candles_tx, mut candles_rx) = mpsc::unbounded();
            let (orderbook_tx, mut orderbook_rx) = mpsc::unbounded();
            let (trades_tx, mut trades_rx) = mpsc::unbounded();

            let mut futures = Vec::new();
            futures.push(self.run_feed(candles_tx, orderbook_tx, trades_tx).boxed());
            futures.push(
                async move {
                    loop {
                        select! {
                            maybe_candles = candles_rx.next() => {
                                if let Some (candles) = maybe_candles {
                                    accumulated = PriceFeedData{ candles: Some(candles), ..accumulated};
                                    state_tx.send(accumulated.clone()).await.expect("Channel expected to be good");
                                } else {
                                    info!("Candles stream finished - exit data feed");
                                }
                            },
                            maybe_orderbook = orderbook_rx.next() =>{
                                if let Some (orderbook) = maybe_orderbook {
                                accumulated = PriceFeedData{ orderbook: Some(orderbook), ..accumulated};
                                state_tx.send(accumulated.clone()).await.expect("Channel expected to be good");
                                } else {
                                    info!("OrderBook stream finished - exit data feed");
                                }
                            }
                            maybe_trades = trades_rx.next() =>{
                                if let Some (trades_aggregate) = maybe_trades {
                                accumulated = PriceFeedData{ trades_aggregate: Some(trades_aggregate), ..accumulated};
                                state_tx.send(accumulated.clone()).await.expect("Channel expected to be good");
                                } else {
                                    info!("Trades stream finished - exit data feed");
                                }
                            }
                        }
                    }
                }
                .boxed(),
            );

            futures::future::join_all(futures).await;

        }
        .boxed()
    }
}
