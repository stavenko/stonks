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

fn tg_price(float: f64) -> String {
    let price_display = format!("{:.2}", float);
    price_display.replace('.', "\\.")
}
impl Signal {
    fn get_ticker_url(&self) -> String {
        match self.price.symbol.source.as_str() {
            "binance" => format!(
                "https://www.binance.com/en/futures/{}",
                self.price.symbol.ticker
            ),
            "bybit" => format!(
                "https://www.bybit.com/trade/usdt/{}/",
                self.price.symbol.ticker
            ),
            "kucoin" => format!(
                "https://www.kucoin.com/futures/trade/{}",
                self.price.symbol.ticker
            ),
            _ => unimplemented!(),
        }
    }

    fn up_or_down(&self)-> &str {
        if self.price.price > self.prev_price {
            "📈 "
        } else {
            "📉"
        }
    }

}

impl Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Обнаружено изменение цены: \n {}:*{}* {} {} {} \\(*{:.2}%*\\)\n [Посмотреть]({}) ",
            self.price.symbol.source,
            self.price.symbol.ticker,
            tg_price(self.prev_price),
            self.up_or_down(),
            tg_price(self.price.price),
            tg_price(self.price.percentage(self.prev_price) * 100.0),
            self.get_ticker_url(),
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
            input.add_url("kucoin", "https://api-futures.kucoin.com");
            input.add_url("bybit", "https://api.bybit.com");
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
