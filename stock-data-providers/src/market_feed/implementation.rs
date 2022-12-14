use app::{mpsc, BoxFuture, FutureExt, Sink, SinkExt, Stream, StreamExt};
use market_feed::{
    candle::Candle, candles::Candles, create_market_feed, fetch_candles, fetch_orderbook,
    order_book::OrderBook, FetchCandlesInput, FetchOrderbookInput, MarketFeedInput,
    MarketFeedMessage, MarketFeedSettings, fetch_historical_trades, FetchHistoricalTradesInput, trade::{Trade, Trades, TradesAggregate},
};
use tracing::{error, info};

use super::{config::PriceFeedConfig, PriceFeed };


impl PriceFeed {
    pub fn new(config: PriceFeedConfig) -> Self {
        let PriceFeedConfig {
            api_key,
            candles,
            orderbook,
            trades,
            api_host,
            ws_host,
            ticker,
        } = config;

        Self {
            api_key,
            candles,
            api_host,
            ws_host,
            ticker,
            orderbook,
            trades,
        }
    }

    pub async fn run_feed(
        self,
        candles_sink: impl Sink<Candles> + Sync + Send + Unpin,
        orderbook_sink: impl Sink<OrderBook> + Sync + Send + Unpin,
        trades_aggregate_sink: impl Sink<TradesAggregate> + Sync + Send + Unpin,
    ) {
        info!(?self, "Init stream");
        let candles = self
            .candles.as_ref()
            .map(|c| MarketFeedSettings::Candle(c.time_unit.clone()));
        let trades = self.trades.as_ref().map(|c| MarketFeedSettings::Trades);
        let orderbook = self.orderbook.as_ref().map(|c| MarketFeedSettings::OrderBook);

        let stream = create_market_feed(MarketFeedInput {
            ticker: self.ticker.clone(),
            settings: vec![candles, trades, orderbook]
                .into_iter()
                .filter_map(|item| item)
                .collect(),
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
        let candles_settings = self.candles.as_ref().unwrap();
        let candles_amount = candles_settings.amount;
        async move {
            let mut candles = fetched_candles;
            while let Some(candle) = candles_stream.next().await {
                candles.join(candle);
                candles.split_on(candles_amount);
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
        if let Some(candles) = self.candles.as_ref() {
            info!("Get candles snapshot");
            let result = fetch_candles(FetchCandlesInput {
                ticker: self.ticker.clone(),
                time_unit: candles.time_unit.clone(),
                api_host: self.api_host.clone(),
                countback: candles.amount,
            })
            .await;

            self.candles_future(result, candles_stream, sink).await
        }
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

    fn trades_future<'f>(
        &self,
        trades_history: Trades,
        mut trades_stream: impl Stream<Item = Trade> + Send + Sync + Unpin + 'f,
        mut sink: impl Sink<TradesAggregate> + Send + Sync + Unpin + 'f,
    ) -> BoxFuture<'f, ()> {
        async move {
            let mut snapshot = trades_history;
            while let Some(trade) = trades_stream.next().await {
                snapshot.add(trade);
                let agg:TradesAggregate = snapshot.calculate_aggregate();
                if sink.send(agg.clone()).await.is_err() {
                    error!("Sink must be ok");
                    panic!();
                }
            }
        }
        .boxed()
    }

    async fn run_trades_future<'f>(
        &self,
        trades_stream: impl Stream<Item = Trade> + Send + Sync + Unpin + 'f,
        sink: impl Sink<TradesAggregate> + Send + Sync + Unpin + 'f,
    ) {
        if let Some(trades) = self.trades.as_ref() {
            let result = fetch_historical_trades(FetchHistoricalTradesInput {
                from: trades.from,
                api_key: self.api_key.clone(),
                ticker: self.ticker.clone(),
                api_host: self.api_host.clone(),
            })
            .await;

            self.trades_future(result, trades_stream, sink).await
        }
    }
    async fn run_orderbook_future<'f>(
        &self,
        orderbook_stream: impl Stream<Item = OrderBook> + Send + Sync + Unpin + 'f,
        sink: impl Sink<OrderBook> + Send + Sync + Unpin + 'f,
    ) {
        if let Some(ob) = self.orderbook.as_ref() {
            let result = fetch_orderbook(FetchOrderbookInput {
                ticker: self.ticker.clone(),
                depth: ob.depth,
                api_host: self.api_host.clone(),
            })
            .await;

            self.orderbook_future(result, orderbook_stream, sink).await
        }
    }
}
