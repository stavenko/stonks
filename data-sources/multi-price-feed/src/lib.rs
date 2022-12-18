use std::{collections::HashMap, sync::Arc, time::Duration};

use binance::fut::fetch_ticker_price;
use futures::{channel::mpsc, FutureExt, SinkExt, Stream};
use tracing::{info, warn};
use url::Url;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Symbol {
    pub ticker: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Price {
    pub symbol: Symbol,
    pub price: f64,
}

impl Price {
    pub fn percentage(&self, prev: f64) -> f64 {
        let diff = (prev - self.price).abs();
        diff / prev
    }
}

type FilterClosure<T> = Arc<dyn Fn(&T) -> bool + Send + Sync>;

#[derive(Clone)]
pub struct GetMultiPriceFeedInput {
    urls: HashMap<String, Url>,
    binance_filters: Vec<FilterClosure<Symbol>>,
    waiting_period: Duration,
}

impl GetMultiPriceFeedInput {
    pub fn new(period: Duration) -> Self {
        GetMultiPriceFeedInput {
            urls: Default::default(),
            binance_filters: Default::default(),
            waiting_period: period,
        }
    }
    pub fn add_filter(&mut self, filter: impl Fn(&Symbol) -> bool + Send + Sync + 'static) {
        self.binance_filters.push(Arc::new(filter));
    }

    pub fn add_url(&mut self, source: &str, url: &str) {
        self.urls.insert(source.into(), Url::parse(url).unwrap());
    }
}

pub async fn get_multi_price_feed(
    input: GetMultiPriceFeedInput,
) -> impl Stream<Item = Price> + Send + Sync {
    let (tx, rx) = mpsc::unbounded();
    let mut futures = Vec::new();

    #[cfg(feature = "binance")]
    {
        let mut tx = tx.clone();
        let symbols = collect_symbols(input.clone())
            .await
            .into_iter()
            .map(|symbol| (symbol.ticker.clone(), symbol))
            .collect::<HashMap<_, _>>();
        info!(?symbols, "Collected symbols for monitoring");
        let waiting_period = input.waiting_period;
        futures.push(
            async move {
                let api_host = "https://fapi.binance.com";
                let url = input
                    .urls
                    .get("binance")
                    .map(Clone::clone)
                    .unwrap_or(Url::parse(api_host).unwrap());
                loop {

                    let prices = fetch_ticker_price(url.clone()).await;
                    info!("query prices");
                    for p in prices {
                        if let Some(symbol) = symbols.get(&p.symbol) {
                            let symbol = Symbol {
                                ticker: symbol.ticker.clone(),
                                base_asset: symbol.base_asset.clone(),
                                quote_asset: symbol.quote_asset.clone(),
                                source: "binance".into(),
                            };
                            let price = Price {
                                symbol,
                                price: p.price,
                            };
                            if let Err(e) = tx.send(price).await {
                                warn!(?e, "Cannot pass price from binance");
                            }
                        }
                    }
                    info!(?waiting_period, "Wait for");
                    tokio::time::sleep(waiting_period).await;
                }
            }
            .boxed(),
        );
    }

    tokio::spawn(async move {
        futures::future::join_all(futures).await;
    });

    rx
}

#[cfg(feature = "binance")]
impl From<binance::fut::exchange_info::Symbol> for Symbol {
    fn from(bs: binance::fut::exchange_info::Symbol) -> Self {
        Self {
            source: "binance".into(),
            ticker: bs.symbol,
            quote_asset: bs.quote_asset,
            base_asset: bs.base_asset,
        }
    }
}

async fn collect_symbols(input: GetMultiPriceFeedInput) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    #[cfg(feature = "binance")]
    {
        use binance::fut::exchange_info::{ExchangeInfoRequest, Symbol as BinanceSymbol};
        let api_host = "https://fapi.binance.com";
        let url = input
            .urls
            .get("binance")
            .map(Clone::clone)
            .unwrap_or(Url::parse(api_host).unwrap());
        let info = binance::fut::fetch_exchange_info(url, ExchangeInfoRequest::default()).await;

        let mut iter: Box<dyn Iterator<Item = Symbol>> = Box::new(
            info.symbols
                .into_iter()
                .map(|v| serde_json::from_value::<BinanceSymbol>(v).unwrap())
                .map(|v| v.into()),
        );

        for f in &input.binance_filters {
            let i = iter.filter(f.as_ref());
            iter = Box::new(i);
        }
        symbols.extend(iter);
    }

    symbols
}

#[cfg(test)]
mod test {
    use crate::collect_symbols;

    #[tokio::test]
    #[cfg(feature = "binance")]
    async fn get_all_symbols() {
        use std::{sync::Arc, time::Duration};

        let mut input = crate::GetMultiPriceFeedInput::new(Duration::from_secs(15));
        input
            .binance_filters
            .push(Arc::new(|s| s.quote_asset == "USDT"));
        let symbols = collect_symbols(input).await;
        assert!(symbols.is_empty())
    }
}
