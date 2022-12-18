use std::{collections::HashMap, fmt::Display, time::Duration};

use app::{
    worker::{ProducerWorker, Worker},
    FutureExt, SinkExt, StreamExt,
};
use multi_price_feed::{GetMultiPriceFeedInput, Price, Symbol};
use tracing::info;
use url::Url;

#[derive(Debug)]
pub struct PriceCollector {
    threshold: f64,
    period: Duration,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Direction {
    Long,
    Short,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Signal {
    direction: Direction,
    price: Price,
    prev_price: f64,
}

impl Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Обнаружено изменение цены: {}:{} {} -> {} ({})",
            self.price.symbol.source,
            self.price.symbol.ticker,
            self.prev_price,
            self.price.price,
            self.price.percentage(self.prev_price),
        )
    }
}

impl Eq for Signal {}

impl PriceCollector {
    pub fn new(threshold: f64, period: Duration) -> Self {
        PriceCollector { threshold, period }
    }
}

impl<'f> ProducerWorker<'f, Signal> for PriceCollector {
    fn work(self: Box<Self>, mut state_tx: app::mpsc::Sender<Signal>) -> app::BoxFuture<'f, ()> {
        async move {
            let mut price_storage: HashMap<Symbol, f64> = HashMap::new();
            let mut input = GetMultiPriceFeedInput::new(self.period);
            input.add_filter(|sym| sym.quote_asset.to_uppercase() == "USDT");
            input.add_url("binance", "https://fapi.binance.com");
            let mut price_feed = multi_price_feed::get_multi_price_feed(input).await;

            while let Some(item) = price_feed.next().await {
                if let Some(entry) = price_storage.get_mut(&item.symbol) {
                    let diff = item.percentage(*entry);

                    if diff > self.threshold {
                        let direction = if *entry > item.price {
                            Direction::Short
                        } else {
                            Direction::Long
                        };
                        let signal = Signal {
                            direction,
                            price: item.clone(),
                            prev_price: *entry,
                        };
                        info!(?signal, diff, "report");
                        state_tx.send(signal).await.ok();
                    }
                    *entry = item.price
                } else {
                    info!(?item, "Add price item");
                    price_storage.insert(item.symbol, item.price);
                }
            }
        }
        .boxed()
    }
}
