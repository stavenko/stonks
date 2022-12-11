use futures::{channel::mpsc, future::BoxFuture};
use tokio::sync::watch;

pub trait Worker<'f, Consumed, Produced> {
    fn work(
        self: Box<Self>,
        state_rx: watch::Receiver<Option<Consumed>>,
        state_tx: mpsc::Sender<Produced>,
    ) -> BoxFuture<'f, ()>;
}

pub trait ProducerWorker<'f, T> {
    fn work(
        self: Box<Self>,
        state_tx: mpsc::Sender<T>,
    ) -> BoxFuture<'f, ()>;
}

pub trait ConsumerWorker<'f, T> {
    fn work(
        self: Box<Self>,
        state_rx: watch::Receiver<Option<T>>,
    ) -> BoxFuture<'f, ()>;
}
