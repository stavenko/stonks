use std::time::SystemTime;

use app::{worker::Worker, FutureExt, SinkExt};
use market_feed::{candles::Candles, order_book::OrderBook};
use serde::Deserialize;
use tracing::error;

pub struct Predictor {
    volume_weight_threshold: f64,
    ticker: String,
}

#[derive(Deserialize)]
pub struct PredictorConfig {
    pub volume_weight_threshold: f64,
    pub ticker: String,
}

pub type WorkerInput = (Candles, OrderBook);

#[derive(Debug, Clone)]
pub enum Position {
    Short,
    Long,
}

#[derive(Debug, Clone)]
pub struct TradeSignal {
    pub time: SystemTime,
    pub position: Position,
    pub ticker: String,
}

impl Predictor {
    pub fn new(config: PredictorConfig) -> Self {
        Self {
            ticker: config.ticker,
            volume_weight_threshold: config.volume_weight_threshold,
        }

    }
}

impl Worker<WorkerInput, TradeSignal> for Predictor {
    fn work<'f>(
        self: Box<Self>,
        mut state_rx: tokio::sync::watch::Receiver<Option<WorkerInput>>,
        mut state_tx: app::mpsc::Sender<TradeSignal>,
    ) -> app::BoxFuture<'f, ()> {
        async move {
            loop {
                let mut signal = None;
                if state_rx.changed().await.is_ok() {
                    let borrowed = state_rx.borrow();
                    if let Some((candles, _)) = borrowed.as_ref() {
                        let min = candles.min_low();
                        let max = candles.max_high();
                        let current = candles.current();
                        let volume_weight = candles.last_volume_weight();
                        let ticker = self.ticker.clone();

                        signal = if current < min && volume_weight > self.volume_weight_threshold {
                            Some(TradeSignal {
                                time: SystemTime::now(),
                                position: Position::Short,
                                ticker,
                            })
                        } else if current > max && volume_weight > self.volume_weight_threshold {
                            Some(TradeSignal {
                                time: SystemTime::now(),
                                position: Position::Long,
                                ticker,
                            })
                        } else {
                            None
                        }
                    }
                }
                if let Some(signal) = signal.take() {
                    if let Err(error) = state_tx.send(signal).await {
                        error!(?error, "Error send trade signal");
                    }
                }
            }
        }
        .boxed()
    }
}
