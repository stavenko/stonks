use reqwest::Client;
use serde::Deserialize;
use toolset::deser_float_from_string;
use url::Url;

#[derive(Deserialize)]
pub struct Response<T> {
    #[serde(rename = "ret_code")]
    _code: u16,
    #[serde(rename = "ret_msg")]
    _ret_msg: String,
    result: T,
}

#[derive(Deserialize)]
pub struct Symbol {
    pub name: String,
    pub base_currency: String,
    pub quote_currency: String,
}

#[derive(Deserialize)]
pub struct TickerInfo {
    pub symbol: String,
    #[serde(deserialize_with = "deser_float_from_string")]
    pub last_price: f64,
}

pub async fn fetch_symbols(api_host: Url) -> Vec<Symbol> {
    let url = api_host.join("/v2/public/symbols").unwrap();
    let result = Client::new()
        .get(url.clone())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    match serde_json::from_str::<Response<Vec<Symbol>>>(&result) {
        Ok(r) => r.result,
        Err(e) => {
            panic!("Serde! {e}\n{} {}", url, result);
        }
    }
}

pub async fn fetch_tickers(api_host: Url) -> Vec<TickerInfo> {
    let url = api_host.join("/v2/public/tickers").unwrap();
    let result = Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    match serde_json::from_str::<Response<Vec<TickerInfo>>>(&result) {
        Ok(r) => r.result,
        Err(e) => {
            panic!("Serde! {e}");
        }
    }
}
