use std::cmp::Ordering;

use crate::candle::Candle;
use tracing::info;

#[derive(Default, Debug, Clone)]
pub struct Candles(Vec<Candle>);


impl Candles {
    pub fn new(candles: Vec<Candle>) -> Self {
        Self(candles)
    }
    pub fn join(&mut self, candle: Candle) {
        let Some(last) = self.0.last_mut() else {
            self.0.push(candle);
            return;
        };
        match last.ts.cmp(&candle.ts) {
            Ordering::Equal => {*last = candle;}
            Ordering::Less => {self.0.push(candle);}
            Ordering::Greater => {
                info!(?candle, "Ignore too old candle");
            }
        }
    }

    pub fn split_on(&mut self, to_size: usize) -> Vec<Candle> {
        let left_off = self.0.split_off(to_size);
        std::mem::replace(&mut self.0, left_off)
    }
}

