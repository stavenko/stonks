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
    pub fn new(trades: Vec<Trade>) -> Self{
        Trades(trades)
    }
    pub fn add(&mut self, _trade: Trade) {

    }

    pub fn calculate_aggregate(&self) -> TradesAggregate {
        // TODO: Implement trades aggregate
        TradesAggregate
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct TradesAggregate;
