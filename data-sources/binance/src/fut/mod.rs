use reqwest::Url;

use self::{
    exchange_info::{ExchangeInfo, ExchangeInfoRequest},
    ticker_price::SymbolPrice,
};
pub mod exchange_info;
pub mod ticker_price;

pub async fn fetch_ticker_price(api_host: Url) -> Vec<SymbolPrice> {
    let url = api_host.join("/fapi/v1/ticker/price").unwrap();

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
pub async fn fetch_exchange_info(
    api_host: Url,
    exchange_info: ExchangeInfoRequest,
) -> ExchangeInfo {
    let mut url = api_host.join("/fapi/v1/exchangeInfo").unwrap();
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
