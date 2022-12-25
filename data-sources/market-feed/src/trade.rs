use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct Trade {
    pub price: f64,
    pub quantity: f64,
    pub quote_quantity: f64,
    pub time: Duration, // Trade executed timestamp, as same as `T` in the stream
}

pub struct Trades(Vec<Trade>);

impl Trades {
    pub fn new(trades: Vec<Trade>) -> Self {
        Trades(trades)
    }
    pub fn add(&mut self, _trade: Trade) {}

    pub fn calculate_aggregate(&self, tolerance: f64) -> TradesAggregate {
        // TODO: Implement trades aggregate
        let current_price = self.0.last().unwrap().price;
        let min_price = self.0.iter().min_by(|t1, t2| t1.price.total_cmp(&t2.price)).unwrap().price;
        let max_price = self.0.iter().max_by(|t1, t2| t1.price.total_cmp(&t2.price)).unwrap().price;
        let range = max_price - min_price;
        let tolerance = tolerance * range;
        let min_price = current_price - tolerance / 2.0;
        let max_price = min_price + tolerance;

        let mut total_volume = 0.0;
        let mut price_volume = 0.0;
        for t in &self.0 {
            total_volume += t.quantity;
            if t.price > min_price && t.price < max_price {
                price_volume += t.quantity;
            }
        }

        TradesAggregate {
            support_volume: price_volume / total_volume,
            min_price,
            max_price,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq )]
pub struct TradesAggregate {
    pub support_volume: f64,
    pub min_price: f64,
    pub max_price: f64,
}

impl Eq for TradesAggregate {}
