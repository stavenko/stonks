use app::{mpsc, BoxFuture, FutureExt, Sink, SinkExt, Stream, StreamExt};
use market_feed::{
    candle::Candle, candles::Candles, create_market_feed, fetch_candles, fetch_orderbook,
    order_book::OrderBook, FetchCandlesInput, FetchOrderbookInput, MarketFeedInput,
    MarketFeedMessage,
};
use tracing::{error, info};

use super::{config::PriceFeedConfig, PriceFeed};

impl PriceFeed {
    pub fn new(config: PriceFeedConfig) -> Self {
        let PriceFeedConfig {
            api_host,
            ws_host,
            time_unit,
            ticker,
            orderbook_depth,
        } = config;

        Self {
            api_host,
            ws_host,
            ticker,
            time_unit,
            orderbook_depth,
        }
    }

    pub async fn run_feed(
        self,
        candles_sink: impl Sink<Candles> + Sync + Send + Unpin,
        orderbook_sink: impl Sink<OrderBook> + Sync + Send + Unpin,
    ) {
        info!(?self, "Init stream");
        let stream = create_market_feed(MarketFeedInput {
            ticker: self.ticker.clone(),
            time_unit: self.time_unit.clone(),
            ws_url: self.ws_host.clone(),
        })
        .await;

        info!("Stream connected - init snapshots");

        if let Some(stream) = stream {
            let (mut candle_tx, candle_rx) = mpsc::unbounded();
            let (mut orderbook_tx, orderbook_rx) = mpsc::unbounded();
            let mut futures = Vec::new();

            futures.push(
                async move {
                    let mut stream = stream.boxed();
                    while let Some(item) = stream.next().await {
                        match item {
                            MarketFeedMessage::Candle(c) => {
                                candle_tx.send(c).await.unwrap();
                            }
                            MarketFeedMessage::OrderBook(o) => {
                                orderbook_tx.send(o).await.unwrap();
                            }
                        }
                    }
                }
                .boxed(),
            );

            futures.push(self.run_candles_future(candle_rx, candles_sink).boxed());
            futures.push(
                self.run_orderbook_future(orderbook_rx, orderbook_sink)
                    .boxed(),
            );

            futures::future::join_all(futures).await;
        } else {
            panic!("Failed to init market stream");
        }
    }

    fn candles_future<'f>(
        &self,
        fetched_candles: Candles,
        mut candles_stream: impl Stream<Item = Candle> + Send + Sync + Unpin + 'f,
        mut sink: impl Sink<Candles> + Send + Sync + Unpin + 'f,
    ) -> BoxFuture<'f, ()> {
        async move {
            let mut candles = fetched_candles;
            while let Some(candle) = candles_stream.next().await {
                candles.join(candle);
                if sink.send(candles.clone()).await.is_err() {
                    error!("Sink must be ok");
                    panic!();
                }
            }
        }
        .boxed()
    }

    async fn run_candles_future<'f>(
        &self,
        candles_stream: impl Stream<Item = Candle> + Send + Sync + Unpin + 'f,
        sink: impl Sink<Candles> + Send + Sync + Unpin + 'f,
    ) {
        info!("Get candles snapshot");
        let result = fetch_candles(FetchCandlesInput {
            ticker: self.ticker.clone(),
            time_unit: self.time_unit.clone(),
            api_host: self.api_host.clone(),
        })
        .await;

        self.candles_future(result, candles_stream, sink).await
    }

    fn orderbook_future<'f>(
        &self,
        orderbook_snapshot: OrderBook,
        mut orderbook_stream: impl Stream<Item = OrderBook> + Send + Sync + Unpin + 'f,
        mut sink: impl Sink<OrderBook> + Send + Sync + Unpin + 'f,
    ) -> BoxFuture<'f, ()> {
        async move {
            let mut snapshot = orderbook_snapshot;
            while let Some(ob) = orderbook_stream.next().await {
                snapshot.join(ob);
                if sink.send(snapshot.clone()).await.is_err() {
                    error!("Sink must be ok");
                    panic!();
                }
            }
        }
        .boxed()
    }

    async fn run_orderbook_future<'f>(
        &self,
        orderbook_stream: impl Stream<Item = OrderBook> + Send + Sync + Unpin + 'f,
        sink: impl Sink<OrderBook> + Send + Sync + Unpin + 'f,
    ) {
        // TODO: Query orderbook snapshot;
        let result = fetch_orderbook(FetchOrderbookInput {
            ticker: self.ticker.clone(),
            depth: self.orderbook_depth,
            api_host: self.api_host.clone(),
        })
        .await;

        self.orderbook_future(result, orderbook_stream, sink).await
    }
}
