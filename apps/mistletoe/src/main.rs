use std::fs::File;

use app::{App, InjectedTo};
use clap::Parser;
use market_feed::{candles::Candles, order_book::OrderBook};
use serde::Deserialize;
use stock_data_providers::market_feed::{config::PriceFeedConfig, PriceFeed, PriceFeedData};
use tracing::info;


#[derive(Debug, Clone, Default)]
struct Mistletoe {
    candles: Candles,
    orderbook: OrderBook,
}

#[derive(Deserialize)]
struct Config {
    price_feed: PriceFeedConfig,
}

#[derive(Parser)]
struct Opts {
    #[arg(short, long)]
    config_file: String,
}

impl InjectedTo<Mistletoe> for PriceFeedData {
    fn inject_to(self, _state: Mistletoe) -> Mistletoe {
        Mistletoe { candles: self.candles, orderbook: self.orderbook }
    }
}

async fn runner() {
    let cli_opts = Opts::parse(); 
    tracing_subscriber::fmt::init();

    let config: Config = serde_yaml::from_reader(File::open(cli_opts.config_file).unwrap()).unwrap();
    let app = App::build(Mistletoe::default())
        .add_producer(PriceFeed::new(config.price_feed))
        .build();

    info!("run app");
    app.run().await
}

#[tokio::main]
async fn main() {
    runner().await;
}
