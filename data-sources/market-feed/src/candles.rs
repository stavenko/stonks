use std::{cmp::Ordering, slice::Iter};

use crate::candle::Candle;
use sources_common::time_unit::TimeUnit;
use tracing::info;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Candles(Vec<Candle>);

impl IntoIterator for Candles {
    type Item = Candle;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Candles {
    pub fn iter(&self) -> Iter<Candle> {
        self.0.iter()
    }

    pub fn min_low(&self) -> f64 {
        let min = self.iter().map(|c| c.low).min_by(|a, b| a.total_cmp(b));
        min.unwrap_or(f64::NEG_INFINITY)
    }

    pub fn max_high(&self) -> f64 {
        let min = self.iter().map(|c| c.high).max_by(|a, b| a.total_cmp(b));
        min.unwrap_or(f64::NEG_INFINITY)
    }

    pub fn total_volume(&self) -> f64 {
        self.0.iter().map(|candle| candle.volume).sum()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn latest_candle(&self) -> Option<&Candle> {
        self.0.last()
    }

    pub fn time_unit(&self) -> Option<TimeUnit> {
        self.latest_candle().map(|c| c.time_unit.clone())
    }

    pub fn last_volume_weight(&self) -> f64 {
        let total = self.total_volume();
        let last_volume = self
            .latest_candle()
            .map(|candle| candle.volume)
            .unwrap_or(0.0);
        last_volume / total
    }

    pub fn current(&self) -> f64 {
        self.latest_candle()
            .map(|last_candle| last_candle.close)
            .unwrap_or(0.0)
    }

    pub fn new(candles: Vec<Candle>) -> Self {
        Self(candles)
    }
    pub fn join(&mut self, candle: Candle) {
        let Some(last) = self.0.last_mut() else {
            self.0.push(candle);
            return;
        };
        match last.ts.cmp(&candle.ts) {
            Ordering::Equal => {
                *last = candle;
            }
            Ordering::Less => {
                self.0.push(candle);
            }
            Ordering::Greater => {
                info!(?candle, "Ignore too old candle");
            }
        }
    }

    pub fn split_on(&mut self, to_size: usize) -> Vec<Candle> {
        let split_at = self.0.len() - to_size;
        let left_off = self.0.split_off(split_at);
        std::mem::replace(&mut self.0, left_off)
    }
}
