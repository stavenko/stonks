use core::fmt;
use futures::{channel::mpsc, SinkExt, Stream, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{de::DeserializeOwned, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info};
use url::Url;

use crate::{
    error::Error,
    protocol::{Response, StreamData, StreamPackage, SubscribeMessage},
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
pub mod trade;

fn conversion(text: String) -> Result<StreamData, Error> {
    let stream_data = serde_json::from_str::<StreamData>(&text)
        .map_err(|e| Error::SerdeError(e, text.to_string()))?;

    Ok(stream_data)
}

pub async fn get_market_stream(
    mut ws_host: Url,
    subscribe_streams: Vec<Box<dyn ToChannel + Send>>,
) -> impl Stream<Item = StreamPackage> {
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
                Message::Text(txt) => match conversion(txt) {
                    Ok(StreamData::Package(p)) => tx.send(p).await.unwrap(),
                    Ok(StreamData::SubscribeResponse { response, id }) => {
                        info!(?response, ?id, "SubscribeResponse");
                    }
                    Err(e) => {
                        error!(?e, "Error occured");
                    }
                },
                Message::Close(_) => {
                    tx.close().await.unwrap();
                    ws_tx.close().await.unwrap();
                    break;
                }
                _ => unreachable!("Something unexpected"),
            }
        }
    });

    rx
}

pub async fn fetch<Q, R>(api_host: Url, path: &str, query: Q, headers: Option<HeaderMap>) -> R
where
    Q: Serialize + fmt::Debug,
    R: DeserializeOwned,
{
    let mut url = api_host.join(path).unwrap();
    let qs = serde_qs::to_string(&query).unwrap();

    info!(?query, ?qs, "Run query");

    url.set_query(Some(&qs));
    let mut req = reqwest::Client::new().get(url);
    if let Some(headers) = headers {
        req = req.headers(headers);
    }

    let result = req.send().await.unwrap().text().await.unwrap();

    match serde_json::from_str::<Response<R>>(&result) {
        Ok(Response::Success(t)) => t,
        Ok(Response::Error { code, msg }) => {
            panic!("Bam! {path} {code} ({msg})");
        }

        Err(e) => {
            panic!("Serde! {e} {}", result);
        }
    }
}
pub async fn fetch_type<Q, R>(api_host: Url, path: &str, query: Q, headers: Option<HeaderMap>) -> R
where
    Q: Serialize + fmt::Debug,
    R: DeserializeOwned,
{
    let mut url = api_host.join(path).unwrap();
    let qs = serde_qs::to_string(&query).unwrap();

    info!(?query, ?qs, "Run query");

    url.set_query(Some(&qs));
    let mut req = reqwest::Client::new().get(url);
    if let Some(headers) = headers {
        req = req.headers(headers);
    }

    let result = req.send().await.unwrap().text().await.unwrap();

    match serde_json::from_str::<R>(&result) {
        Ok(t) => t,
        Err(e) => {
            panic!("Serde! {e} {}", result);
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
    fetch(api_host, "/api/v3/klines", candles_query, None).await
}

pub async fn fetch_orderbook(api_host: Url, orderbook_query: OrderBookQuery) -> ApiOrderBook {
    fetch(api_host, "/api/v3/depth", orderbook_query, None).await
}

pub async fn fetch_historical_trades(
    api_host: Url,
    historical_trades_query: HistoricalTradesQuery,
) -> Vec<ApiHistoricalTrade> {
    let mut headers = HeaderMap::new();
    headers.insert("X-MBX-APIKEY", HeaderValue::from_str(&historical_trades_query.api_key).unwrap());
    fetch_type(
        api_host,
        "/api/v3/historicalTrades",
        historical_trades_query.query,
        Some(headers)
    )
    .await
}

pub async fn fetch_all_historical_trades(
    api_host: Url,
    all_historical_trades_query: AllHistoricalTradesQuery,
) -> Vec<ApiHistoricalTrade> {
    let mut trades: Vec<Vec<ApiHistoricalTrade>> = Vec::new();
    info!(?all_historical_trades_query, "Do query");
    let window = all_historical_trades_query.query.window;
    let now = || SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
        if let Some(true) = trades
            .last()
            .and_then(|r| r.first())
            .map(|first_trade| (now() - first_trade.time) > window)
        {
            break;
        }

        let from_id = trades.last().and_then(|t| t.first()).map(|t| t.id - 500);
        let fetched_trades = fetch_historical_trades(
            api_host.clone(),
            HistoricalTradesQuery {
                query: Query {
                    symbol: all_historical_trades_query.query.ticker.clone(),
                    from_id,
                },
                api_key: all_historical_trades_query.api_key.clone(),
            },
        ).await;

        trades.push(fetched_trades);
    }

    trades.into_iter().rev().flatten().collect()
}
