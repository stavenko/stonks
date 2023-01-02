use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tracing::warn;

#[derive(Debug, Clone, PartialEq)]
pub struct Trade {
    pub price: f64,
    pub quantity: f64,
    pub quote_quantity: f64,
    pub time: Duration, // Trade executed timestamp, as same as `T` in the stream
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TradesAggregate {
    pub support_volume: f64,
    pub min_price: f64,
    pub max_price: f64,
    pub current_price: f64,
    pub speed_factor: f64,
}

pub struct AggregateOptions {
    tolerance: f64,
    tick_size: f64,
    speed_factor_window: Duration,
}

pub struct Trades(Vec<Trade>);

impl Trades {
    pub fn new(trades: Vec<Trade>) -> Self {
        Trades(trades)
    }

    pub fn add(&mut self, trade: Trade) {
        self.0.push(trade);
    }

    pub fn remove_old(&mut self, window: Duration) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let was = self.0.len();
        self.0.retain(|t| now - t.time < window);
        let deleted = was - self.0.len();
        warn!("deleted {}/ {}", deleted, self.0.len());
    }

    pub fn calculate_speed_factor(&self, window: Duration) -> Option<f64> {
        self.0
            .first()
            .and_then(|f| self.0.last().map(|l| (f, l)))
            .and_then(|(first_trade, last_trade)| {
                let time_between = last_trade.time - first_trade.time;
                let average_trades_per_second = self.0.len() as f64 / time_between.as_secs_f64();
                let time_mark = SystemTime::now().duration_since(UNIX_EPOCH).unwrap() - window;
                self.0
                    .iter()
                    .position(|trade| trade.time > time_mark)
                    .map(|pos| (pos, self.0.get(pos).unwrap()))
                    .and_then(|f| self.0.last().map(|l| (f, l)))
                    .map(|((position, first_trade), last_trade)| {
                        let time_between = last_trade.time - first_trade.time;
                        let amount_in_interval = self.0.len() - position;
                        amount_in_interval as f64 / time_between.as_secs_f64()
                    })
                    .map(|window_speed| window_speed / average_trades_per_second)
            })
    }

    pub fn calculate_aggregate(&self, options: &AggregateOptions) -> TradesAggregate {
        // TODO: Implement trades aggregate
        let current_price = self.0.last().unwrap().price;
        let mut min_price = f64::INFINITY;
        let mut max_price = f64::NEG_INFINITY;
        for t in &self.0 {
            min_price = min_price.min(t.price);
            max_price = max_price.max(t.price);
        }
        let tolerance = options.tolerance.max(options.tick_size / 2.0);
        let range = max_price - min_price;
        let tolerance = tolerance * range;
        let lower_price = current_price - tolerance / 2.0;
        let higher_price = lower_price + tolerance;

        let speed_factor = self
            .calculate_speed_factor(options.speed_factor_window)
            .unwrap_or(1.0);

        let mut total_volume = 0.0;
        let mut price_volume = 0.0;
        for t in &self.0 {
            total_volume += t.quantity;
            if t.price > lower_price && t.price < higher_price {
                price_volume += t.quantity;
            }
        }

        TradesAggregate {
            support_volume: price_volume / total_volume,
            min_price,
            max_price,
            current_price,
            speed_factor,
        }
    }
}

impl AggregateOptions {
    pub  fn new(speed_factor_window: Duration, tolerance: f64, tick_size: f64) -> Self {
        Self {
            speed_factor_window,
            tolerance,
            tick_size,
        }
    }
}
impl Eq for TradesAggregate {}
