use std::fs::File;

use app::{App, InjectedTo, Reduced};
use clap::Parser;
use market_feed::{candles::Candles, order_book::OrderBook};
use predictor::{TradeSignal, WorkerInput};
use serde::Deserialize;
use stock_data_providers::market_feed::{config::PriceFeedConfig, PriceFeed, PriceFeedData};
use tracing::info;

use crate::predictor::{Predictor, PredictorConfig};

mod predictor;

#[derive(Debug, Clone, Default)]
struct Mistletoe {
    candles: Candles,
    orderbook: OrderBook,
    trade_signals: Vec<TradeSignal>,
}

#[derive(Deserialize)]
struct Config {
    price_feed: PriceFeedConfig,
    predictor: PredictorConfig,
}

#[derive(Parser)]
struct Opts {
    #[arg(short, long)]
    config_file: String,
}

impl InjectedTo<Mistletoe> for PriceFeedData {
    fn inject_to(self, state: Mistletoe) -> Mistletoe {
        Mistletoe {
            candles: self.candles,
            orderbook: self.orderbook,
            ..state
        }
    }
}

impl InjectedTo<Mistletoe> for TradeSignal {
    fn inject_to(self, mut state: Mistletoe) -> Mistletoe {
        state.trade_signals.push(self);
        state
    }
}

impl Reduced<Mistletoe> for WorkerInput {
    fn reduce(state: Mistletoe) -> Self {
        (state.candles, state.orderbook)
    }
}

async fn runner() {
    let cli_opts = Opts::parse();
    tracing_subscriber::fmt::init();

    let config: Config =
        serde_yaml::from_reader(File::open(cli_opts.config_file).unwrap()).unwrap();
    let app = App::build(Mistletoe::default())
        .add_producer(PriceFeed::new(config.price_feed))
        .add_worker(Predictor::new(config.predictor))
        .build();

    info!("run app");
    app.run().await
}

#[tokio::main]
async fn main() {
    runner().await;
}
