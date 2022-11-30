use futures::{channel::mpsc, future::BoxFuture};
use tokio::sync::watch;

pub trait Worker<T> {
    fn work<'f>(
        self: Box<Self>,
        state_rx: watch::Receiver<Option<T>>,
        state_tx: mpsc::Sender<T>,
    ) -> BoxFuture<'f, ()>;
}
