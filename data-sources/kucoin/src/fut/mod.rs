use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct Response<T> {
    code: String,
    data: T,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActiveContract {
    pub symbol: String,
    pub base_currency: String,
    pub quote_currency: String,
    pub last_trade_price: f64,
}

pub async fn fetch_active_contracts(api_host: Url) -> Vec<ActiveContract> {

    let url = api_host.join("/api/v1/contracts/active").unwrap();

    let result = reqwest::Client::new()
        .get(url.clone())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    match serde_json::from_str::<Response<Vec<ActiveContract>>>(&result) {
        Ok(r) => r.data,
        Err(e) => {
            panic!("Serde! {e} wtf: {} \n {:?}", result, url);
        }
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn fetch_prices() {
        let url = Url::parse("https://api-futures.kucoin.com").unwrap();
        let active_contracts = fetch_active_contracts(url).await;
        println!(
            "active_contracts: {:#?}",
            active_contracts
                .iter()
                .take(5)
                .map(Clone::clone)
                .collect::<Vec<_>>()
        );
        assert!(!active_contracts.is_empty())
    }
}
