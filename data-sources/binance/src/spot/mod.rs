use core::fmt;
use futures::{channel::mpsc, SinkExt, Stream, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, info};
use url::Url;

use crate::{
    error::Error,
    protocol::{Response, StreamData, SubscribeMessage},
    ToChannel,
};

use self::{
    candle::{Candle, CandlesQuery},
    exchange_info::{ExchangeInfo, ExchangeInfoRequest},
    historical_trades::{
        AllHistoricalTradesQuery, ApiHistoricalTrade, HistoricalTradesQuery, Query,
    },
    orderbook::{ApiOrderBook, OrderBookQuery},
};

pub mod candle;
pub mod exchange_info;
pub mod historical_trades;
pub mod orderbook;

fn conversion(text: String) -> Result<StreamData, Error> {
    let stream_data = serde_json::from_str::<StreamData>(&text)
        .map_err(|e| Error::SerdeError(e, text.to_string()))?;

    Ok(stream_data)
}

pub async fn get_market_stream(
    mut ws_host: Url,
    subscribe_streams: Vec<Box<dyn ToChannel + Send>>,
) -> impl Stream<Item = Result<StreamData, Error>> {
    ws_host.set_path("/stream");

    let (stream, _response) = connect_async(ws_host).await.unwrap();
    let (mut ws_tx, mut ws_rx) = stream.split();
    let (mut tx, rx) = mpsc::unbounded();

    let channels = subscribe_streams
        .into_iter()
        .map(|c| c.to_channel())
        .collect();
    let command = SubscribeMessage {
        method: "SUBSCRIBE".to_string(),
        params: channels,
        id: 100,
    };

    debug!(?command, "Send command to binance web socket");
    let command_str = serde_json::to_string_pretty(&command).unwrap();
    let command = Message::Text(command_str);
    ws_tx.send(command).await.unwrap();

    tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Ping(v) => ws_tx.send(Message::Pong(v)).await.unwrap(),
                Message::Text(txt) => {
                    tx.send(txt).await.unwrap();
                }
                Message::Close(_) => {
                    tx.close().await.unwrap();
                    ws_tx.close().await.unwrap();
                    break;
                }
                _ => unreachable!("Something unexpected"),
            }
        }
    });

    rx.map(conversion)
}

pub async fn fetch<Q, R>(api_host: Url, path: &str, query: Q) -> R
where
    Q: Serialize + fmt::Debug,
    R: DeserializeOwned,
{
    let mut url = api_host.join(path).unwrap();
    let qs = serde_qs::to_string(&query).unwrap();

    info!(?query, ?qs, "Run query");

    url.set_query(Some(&qs));
    let result = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    match serde_json::from_str::<Response<R>>(&result) {
        Ok(Response::Success(t)) => t,
        Ok(Response::Error { code, msg }) => {
            panic!("Bam! {path} {code} ({msg})");
        }

        Err(e) => {
            panic!("Serde! {e}");
        }
    }
}

pub async fn fetch_exchange_info(
    api_host: Url,
    exchange_info: ExchangeInfoRequest,
) -> ExchangeInfo {
    let mut url = api_host.join("/api/v3/exchangeInfo").unwrap();
    let qs = serde_qs::to_string(&exchange_info).unwrap();
    url.set_query(Some(&qs));
    let result = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    match serde_json::from_str(&result) {
        Ok(r) => r,
        Err(e) => {
            panic!("Serde! {e}");
        }
    }
}

pub async fn fetch_candles(api_host: Url, candles_query: CandlesQuery) -> Vec<Candle> {
    fetch(api_host, "/api/v3/klines", candles_query).await
}

pub async fn fetch_orderbook(api_host: Url, orderbook_query: OrderBookQuery) -> ApiOrderBook {
    fetch(api_host, "/api/v3/depth", orderbook_query).await
}

pub async fn fetch_historical_trades(
    api_host: Url,
    historical_trades_query: HistoricalTradesQuery,
) -> Vec<ApiHistoricalTrade> {
    fetch(
        api_host,
        "/api/v3/historicalTrades",
        historical_trades_query.query,
    )
    .await
}

pub async fn fetch_all_historical_trades(
    api_host: Url,
    all_historical_trades_query: AllHistoricalTradesQuery,
) -> Vec<ApiHistoricalTrade> {
    let mut trades: Vec<Vec<ApiHistoricalTrade>> = Vec::new();
    loop {
        if let Some(true) = trades
            .last()
            .and_then(|r| r.first())
            .map(|first_trade| first_trade.time < all_historical_trades_query.query.from_date)
        {
            break;
        }

        let last_id = trades.last().and_then(|t| t.first()).map(|t| t.id);

        trades.push(
            fetch_historical_trades(
                api_host.clone(),
                HistoricalTradesQuery {
                    query: Query {
                        ticker: all_historical_trades_query.query.ticker.clone(),
                        from_id: last_id,
                    },
                    api_key: all_historical_trades_query.api_key.clone(),
                },
            )
            .await,
        );
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    trades.into_iter().rev().flatten().collect()
}
