#[derive(Clone, Debug, Default)]
pub struct OrderBook {
    pub asks: Vec<[f64; 2]>,
    pub bids: Vec<[f64; 2]>,
}

fn join_arr(state: &mut Vec<[f64; 2]>, update: &[[f64; 2]]) {
    for a in update {
        if a[1] == 0.0 {
            if let Some(ix) = state.iter().position(|ask| ask[0] == a[0]) {
                state.remove(ix);
            }
        } else if let Some(pos) = state.iter_mut().find(|ask| ask[0] == a[0]) {
            pos[1] = a[1];
        } else {
            state.push(*a);
            state.sort_by(|a, b| a[0].partial_cmp(&b[0]).expect("wrong float in order book") );
        }
    }
}

impl OrderBook {
    pub fn join(&mut self, ob: Self) {
        join_arr(&mut self.asks, &ob.asks);
        join_arr(&mut self.bids, &ob.bids);
    }
}
