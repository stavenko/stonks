use std::fs::File;

use app::{App, InjectedTo, Reduced};
use clap::Parser;
use market_feed::{candles::Candles, order_book::OrderBook};
use predictor::{PredictorSignal, WorkerInput};
use serde::Deserialize;
use stock_data_providers::market_feed::{config::PriceFeedConfig, PriceFeed, PriceFeedData};
use tg_reporter::{TelegramReporterConfig, TelegramReporter};
use tracing::info;

use crate::predictor::{Predictor, PredictorConfig};

mod predictor;

#[derive(Debug, Clone, Default, Eq, PartialEq,)]
struct Mistletoe {
    candles: Candles,
    orderbook: OrderBook,
    trade_signals: Vec<PredictorSignal>,
}

#[derive(Deserialize)]
struct Config {
    price_feed: PriceFeedConfig,
    predictor: PredictorConfig,
    tg: TelegramReporterConfig,
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

impl InjectedTo<Mistletoe> for PredictorSignal {
    fn inject_to(self, mut state: Mistletoe) -> Mistletoe {
        state.trade_signals.push(self);
        state
    }
}

impl Reduced<WorkerInput> for Mistletoe {
    fn reduce(&mut self) -> WorkerInput {
        (self.candles.clone(), self.orderbook.clone())
    }
}
/*
impl Reduced<Mistletoe> for WorkerInput {
    fn reduce(state: Mistletoe) -> Self {
        (state.candles, state.orderbook)
    }
}
*/

impl Reduced<Vec<PredictorSignal>> for Mistletoe {
    fn reduce(&mut self) -> Vec<PredictorSignal> {
        std::mem::replace(&mut self.trade_signals, Vec::new())
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
        .add_consumer(TelegramReporter::new(config.tg))
        .build();

    info!("run app");
    app.run().await
}

#[tokio::main]
async fn main() {
    runner().await;
}
