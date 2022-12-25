use std::collections::{BTreeMap, VecDeque};

pub struct MinMax<T>(BTreeMap<T, u32>);

pub struct Histogramm<T> {
    min_max: MinMax<T>,
}


impl<T> Histogramm<T> 
where T: Ord + Clone
{
    pub fn insert(&mut self, value: T) {
        self.min_max.add(value);
    }

    pub fn remove(&mut self, value: &T) {
        self.min_max.sub(&value);
    }
}



impl<T> MinMax<T>
where
    T: Ord,
{
    pub fn min(&self) -> Option<&T> {
        self.0.iter().next().map(|(f, _)| f)
    }
    pub fn max(&self) -> Option<&T> {
        self.0.iter().rev().next().map(|(f, _)| f)
    }
    pub fn add(&mut self, value: T) {
        self.0.entry(value).and_modify(|v| *v += 1).or_insert(0);
    }

    pub fn sub(&mut self, value: &T) {
        self.0
            .get_mut(value)
            .map(|v| *v = v.checked_sub(1).expect("min max must be initialized"));

        if let Some(0) = self.0.get(value) {
            self.0.remove(value);
        }
    }
}
