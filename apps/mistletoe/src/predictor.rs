use std::fmt;
use std::time::SystemTime;

use app::{worker::Worker, FutureExt, SinkExt, mpsc, StreamExt};
use chrono::Utc;
use market_feed::{candles::Candles, order_book::OrderBook};
use serde::Deserialize;
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;
use tracing::{error, info};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Position {
    Short,
    Long,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredictorSignal {
    TradeSignal {
        time: SystemTime,
        position: Position,
        ticker: String,
    },
    IAmOk,
}

impl fmt::Display for PredictorSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PredictorSignal::IAmOk => {
                write!(f, "I am okey")
            }
            PredictorSignal::TradeSignal {
                time,
                position,
                ticker,
            } => {
                let time: chrono::DateTime<Utc> = (*time).into();
                write!(
                    f,
                    "{} Open position *{:?}* for ticker {}",
                    time.format("%H:%M"),
                    position,
                    ticker
                )
            }
        }
    }
}

impl Predictor {
    pub fn new(config: PredictorConfig) -> Self {
        Self {
            ticker: config.ticker,
            volume_weight_threshold: config.volume_weight_threshold,
        }
    }

    fn calculate_signal(
        &self,
        candles: &Candles,
        sent_staff: &mut SystemTime,
    ) -> Option<PredictorSignal> {
        let min = candles.min_low();
        let max = candles.max_high();
        let current = candles.current();
        let volume_weight = candles.last_volume_weight();
        let ticker = self.ticker.clone();

        // Use distance from average as indicator
        // Use normalized volume in last candle - volume in moment as it will be in

        if current < min && volume_weight > self.volume_weight_threshold {
            return Some(PredictorSignal::TradeSignal {
                time: SystemTime::now(),
                position: Position::Short,
                ticker,
            });
        } else if current > max && volume_weight > self.volume_weight_threshold {
            return Some(PredictorSignal::TradeSignal {
                time: SystemTime::now(),
                position: Position::Long,
                ticker,
            });
        } else {
            info!(
                "no-signal {:.4}/{}  {:.4} < {:.4} < {:.4} ({})",
                volume_weight,
                self.volume_weight_threshold,
                min,
                current,
                max,
                candles.len()
            );
            if let Some(time_unit) = candles.time_unit() {
                let candle_dur = time_unit.calc_n(1);
                let time_left = SystemTime::now().duration_since(*sent_staff).unwrap();
                info!("candle_dur {:?}, time_left {:?}", candle_dur, time_left);
                if time_left > candle_dur {
                    info!("WTF");
                    *sent_staff = SystemTime::now();
                    return Some(PredictorSignal::IAmOk);
                }
            }
            None
        }
    }
}

impl<'f> Worker<'f, WorkerInput, Option<WorkerInput>, PredictorSignal, app::mpsc::SendError> for Predictor {
    type InputSink = mpsc::Sender<WorkerInput>;
    type InputStream = WatchStream<Option<WorkerInput>>;

    fn provide_input_stream(&self) -> (Self::InputSink, Self::InputStream) {
        let (tx, rx) = watch::channel(None);
        let (mp_tx, mut mp_rx) = mpsc::channel(1);
        tokio::spawn(async move {
            while let Some(item) = mp_rx.next().await {
                tx.send(Some(item)).ok();
            }
        });

        (mp_tx, WatchStream::new(rx))
    }

    fn work(
        self: Box<Self>,
        mut state_rx: Self::InputStream,
        mut state_tx: app::mpsc::Sender<PredictorSignal>,
    ) -> app::BoxFuture<'f, ()> {
        async move {
            let mut sent_staff = SystemTime::now();
            loop {
                let mut signal = None;
                if let Some(borrowed) = state_rx.next().await {
                    // let borrowed = state_rx.borrow();
                    if let Some((candles, _)) = borrowed.as_ref() {
                        signal = self.calculate_signal(candles, &mut sent_staff);
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
