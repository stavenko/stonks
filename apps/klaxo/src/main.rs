use std::{fs::File, time::Duration};

use app::{App, InjectedTo, Reduced};
use clap::Parser;
use serde::Deserialize;
use tg_reporter::{TelegramReporter, TelegramReporterConfig};
use tracing::info;
use worker::Signal;

use crate::worker::PriceCollector;

mod worker;

#[derive(Deserialize)]
struct Config {
    threshold: f64,
    #[serde(with = "humantime_serde")]
    period: Duration,
    tg: TelegramReporterConfig,
}

#[derive(Default, Debug, PartialEq, Clone, Eq)]
struct Klaxo {
    signals: Vec<Signal>,
}

#[derive(Parser)]
struct Opts {
    #[arg(short, long)]
    config_file: String,
}

impl InjectedTo<Klaxo> for Signal {
    fn inject_to(self, mut state: Klaxo) -> Klaxo {
        state.signals.push(self);
        state
    }
}

impl Reduced<Vec<Signal>> for Klaxo {
    fn reduce(&mut self) -> Vec<Signal> {
        std::mem::take(&mut self.signals)
    }
}

async fn runner() {
    println!("ADASDFAS");
    let cli_opts = Opts::parse();
    tracing_subscriber::fmt::init();

    let config: Config =
        serde_yaml::from_reader(File::open(cli_opts.config_file).unwrap()).unwrap();
    let app = App::build(Klaxo::default())
        .add_producer(PriceCollector::new(config.threshold, config.period))
        .add_consumer(TelegramReporter::new(config.tg))
        .build();

    info!("run app");
    app.run().await
}

#[tokio::main]
async fn main() {
    runner().await;
}
