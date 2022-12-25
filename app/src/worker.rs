use core::fmt;

use futures::{channel::mpsc, future::BoxFuture, Sink, Stream};
use tokio::sync::watch;

pub trait Worker<'f, Consumed, II, Produced, E>
where
    E: fmt::Debug,
{
    type InputStream: Stream<Item = II> + Send + Sync + Unpin;
    type InputSink: Sink<Consumed, Error = E> + Send + Sync + Unpin;

    fn provide_input_stream(&self) -> (Self::InputSink, Self::InputStream);

    fn work(
        self: Box<Self>,
        state_rx: Self::InputStream,
        state_tx: mpsc::Sender<Produced>,
    ) -> BoxFuture<'f, ()>;
}

pub trait ProducerWorker<'f, T> {
    fn work(self: Box<Self>, state_tx: mpsc::Sender<T>) -> BoxFuture<'f, ()>;
}

pub trait ConsumerWorker<'f, T, E>
where
    E: fmt::Debug,
{
    type Stream: Stream<Item = T> + Send + Sync + Unpin;
    type Sink: Sink<T, Error = E> + Send + Sync + Unpin;
    fn provide_input_stream(&self) -> (Self::Sink, Self::Stream);
    fn work(self: Box<Self>, state_rx: Self::Stream) -> BoxFuture<'f, ()>;
}
