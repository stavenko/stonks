use core::fmt;
use std::{marker::PhantomData, pin::Pin, sync::Arc};

use async_trait::async_trait;
pub use futures::{
    channel::mpsc, future::BoxFuture, stream::FuturesUnordered, FutureExt, SinkExt, StreamExt,
};
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;
use tracing::{error, info};

pub mod worker;

pub struct Worker<S, T>
where
    S: Send + Sync,
{
    instance: Box<dyn worker::Worker<T> + Send>,
    reducer: Box<dyn Fn(S) -> T + Send>,
    inducer: Box<dyn Fn(S, T) -> S + Send>,
}

impl<S, T> Worker<S, T>
where
    S: Send + Sync,
{
    pub fn new(
        instance: impl worker::Worker<T> + Send + 'static,
        reducer: impl Fn(S) -> T + Send + 'static,
        inducer: impl Fn(S, T) -> S + Send + 'static,
    ) -> Self {
        Worker {
            instance: Box::new(instance),
            reducer: Box::new(reducer),
            inducer: Box::new(inducer),
        }
    }
}

pub trait Reduced<S> {
    fn reduce(state: S) -> Self;
    fn induce(self, state: S) -> S;
}

pub struct App<'f, S> {
    state: watch::Receiver<S>,
    state_updater: BoxFuture<'f, ()>,
    runners: Vec<Box<dyn FnOnce(watch::Receiver<S>) -> BoxFuture<'f, ()>>>,
}

pub struct AppBuilder<'f, S> {
    runners: Vec<Box<dyn FnOnce(watch::Receiver<S>) -> BoxFuture<'f, ()>>>,
    state_updater: BoxFuture<'f, ()>,
    state_tx: mpsc::Sender<S>,
    state_rx: watch::Receiver<S>,
}

impl<'f, S> App<'f, S>
where
    S: fmt::Debug + Send + 'f + Sync,
{
    pub fn build(initial_state: S) -> AppBuilder<'f, S> {
        let (state_tx, mut state_rx) = mpsc::channel(1);
        let (wstate_tx, wstate_rx) = watch::channel(initial_state);
        let state_updater = async move {
            while let Some(new_state) = state_rx.next().await {
                wstate_tx.send(new_state).unwrap();
            }
        }
        .boxed();
        AppBuilder {
            runners: Vec::new(),
            state_updater,
            state_tx: state_tx,
            state_rx: wstate_rx,
        }
    }

    pub async fn run(self) {
        let mut futures = FuturesUnordered::new();
        futures.push(self.state_updater);
        for runner in self.runners {
            futures.push(runner(self.state.clone()));
        }

        while futures.next().await.is_some() {}
    }
}

impl<'f, AppState> AppBuilder<'f, AppState>
where
    AppState: Clone + Send + Sync + 'static,
{
    pub fn add_worker<WorkerState>(
        mut self,
        worker: impl worker::Worker<WorkerState> + Send + Sync + 'static,
    ) -> Self
    where
        WorkerState: Reduced<AppState> + fmt::Debug + Send + Sync + 'static,
    {
        let mut state_update = self.state_tx.clone();
        let worker = Box::new(worker);
        let runner = |state_channel: watch::Receiver<AppState>| {
            let (mut reducer_tx, mut reducer_rx) = mpsc::channel(1);
            let (inducer_tx, mut inducer_rx) = mpsc::channel::<WorkerState>(1);

            let mut state_stream = WatchStream::new(state_channel.clone());

            tokio::spawn(async move {
                while let Some(data) = state_stream.next().await {
                    let new_data = WorkerState::reduce(data);
                    reducer_tx
                        .send(new_data)
                        .await
                        .expect("Cannot send data to channel");
                }
            });

            tokio::spawn(async move {
                while let Some(data) = inducer_rx.next().await {
                    info!("Got some data");
                    let prev_state = state_channel.borrow().clone();
                    let new_state = data.induce(prev_state);
                    state_update
                        .send(new_state)
                        .await
                        .expect("Cannot send data to channel");
                }
            });

            let (reduced_state_tx, reduced_state_rx) = tokio::sync::watch::channel(None);

            tokio::spawn(async move {
                while let Some(item) = reducer_rx.next().await {
                    reduced_state_tx.send(Some(item)).ok();
                }
            });

            async move { worker.work(reduced_state_rx, inducer_tx).await }.boxed()
        };

        self.runners.push(Box::new(runner));
        self
    }

    pub fn build(self) -> App<'f, AppState> {
        App {
            state: self.state_rx,
            runners: self.runners,
            state_updater: self.state_updater,
        }
    }
}
