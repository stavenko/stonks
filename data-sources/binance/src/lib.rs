use core::fmt;

use candle::{Candle, CandlesQuery};
use error::Error;
use futures::{channel::mpsc, SinkExt, Stream, StreamExt};
use orderbook::{ApiOrderBook, OrderBookQuery};
use serde::{de::DeserializeOwned, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, info};
use url::Url;

use crate::protocol::{StreamData, SubscribeMessage, Response};

pub mod candle;
pub mod error;
pub mod orderbook;
pub mod protocol;

pub trait ToChannel {
    fn to_channel(&self) -> String;
}

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

    let result = reqwest::get(url).await.unwrap().text().await.unwrap();

    match serde_json::from_str::<Response<R>>(&result) {
        Ok(Response::Success(t)) => t,
        Ok(Response::Error { code, msg }) => {
            panic!("Bam! {path} {code} ({msg})");
        }

        Err(e) => {
            panic!("Serde! {e}, {path} {result}");
        }
    }
}

pub async fn fetch_candles(api_host: Url, candles_query: CandlesQuery) -> Vec<Candle> {
    fetch(api_host, "/api/v3/klines", candles_query).await
}

pub async fn fetch_orderbook(api_host: Url, orderbook_query: OrderBookQuery) -> ApiOrderBook {
    fetch(api_host, "/api/v3/depth", orderbook_query).await
}
