use futures::{channel::mpsc, future::BoxFuture};
use tokio::sync::watch;

pub trait Worker<Consumed, Produced> {
    fn work<'f>(
        self: Box<Self>,
        state_rx: watch::Receiver<Option<Consumed>>,
        state_tx: mpsc::Sender<Produced>,
    ) -> BoxFuture<'f, ()>;
}

pub trait ProducerWorker<T> {
    fn work<'f>(
        self: Box<Self>,
        state_tx: mpsc::Sender<T>,
    ) -> BoxFuture<'f, ()>;
}

pub trait ConsumerWorker<T> {
    fn work<'f>(
        self: Box<Self>,
        state_rx: watch::Receiver<Option<T>>,
    ) -> BoxFuture<'f, ()>;
}
