use core::fmt;

pub use futures::{
    channel::mpsc, future::BoxFuture, stream::FuturesUnordered, FutureExt, SinkExt, StreamExt,
};
pub use futures::{Sink, Stream};
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;
use tracing::{debug, error, info};

pub mod worker;

pub trait Reduced<T> {
    fn reduce(&mut self) -> T;
}

pub trait InjectedTo<S> {
    fn inject_to(self, state: S) -> S;
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
            state_tx,
            state_rx: wstate_rx,
        }
    }

    pub async fn run(self) {
        let mut futures = FuturesUnordered::new();
        futures.push(self.state_updater);
        for runner in self.runners {
            futures.push(runner(self.state.clone()));
        }

        let total_len = futures.len();
        info!(?total_len, "run futures");

        while futures.next().await.is_some() {}
        info!("all futures returned");
    }
}

impl<'f, AppState> AppBuilder<'f, AppState>
where
    AppState: Eq + Clone + Send + Sync + 'static,
{
    async fn state_reducer<WorkerState, E>(
        mut state_stream: impl Stream<Item = AppState> + Unpin,
        mut state_sink: impl Sink<AppState> + Unpin,
        mut consumer: impl Sink<WorkerState, Error = E> + Send + Unpin,
    ) where
        WorkerState: Send + fmt::Debug,
        E: fmt::Debug + Send + Sync,
        AppState: Reduced<WorkerState>,
    {
        while let Some(mut state) = state_stream.next().await {
            let old_state = state.clone(); // this seems like a wrong idea here
            let worker_data = state.reduce();

            consumer
                .send(worker_data)
                .await
                .expect("Cannot send data to channel");
            if old_state != state {
                state_sink.send(state).await;
            }
        }
    }

    async fn state_injector<WorkerState, SinkError>(
        mut provider_stream: impl Stream<Item = WorkerState> + Unpin,
        app_state_watch: watch::Receiver<AppState>,
        mut app_state_sink: impl Sink<AppState, Error = SinkError> + Unpin,
    ) where
        WorkerState: InjectedTo<AppState> + fmt::Debug,
        SinkError: fmt::Debug,
    {
        while let Some(data) = provider_stream.next().await {
            debug!(?data, "Got some data");

            let prev_state = app_state_watch.borrow().clone();
            let new_state = data.inject_to(prev_state);
            app_state_sink
                .send(new_state)
                .await
                .expect("Cannot send data to channel");
        }
    }

    pub fn add_producer<WorkerState>(
        mut self,
        worker: impl worker::ProducerWorker<'f, WorkerState> + Send + Sync + 'static,
    ) -> Self
    where
        WorkerState: InjectedTo<AppState> + fmt::Debug + Send + Sync + 'f,
    {
        let state_update = self.state_tx.clone();
        let worker = Box::new(worker);
        let runner = |state_channel: watch::Receiver<AppState>| {
            let mut data_futures = Vec::new();
            let (inducer_tx, inducer_rx) = mpsc::channel::<WorkerState>(100);
            // `state_reducer and `state_mapper` propagate state to each consumer
            data_futures
                .push(Self::state_injector(inducer_rx, state_channel, state_update).boxed());
            data_futures.push(worker.work(inducer_tx));

            async move {
                futures::future::join_all(data_futures).await;
            }
            .boxed()
        };

        self.runners.push(Box::new(runner));
        self
    }

    pub fn add_consumer<WorkerState, E>(
        mut self,
        worker: impl worker::ConsumerWorker<'f, WorkerState, E> + Send + Sync + 'static,
    ) -> Self
    where
        WorkerState: fmt::Debug + Send + Sync + 'f,
        E: fmt::Debug + Send + Sync + 'f,
        AppState: Reduced<WorkerState>,
    {
        let worker = Box::new(worker);
        let state_tx = self.state_tx.clone();
        let runner = |state_channel: watch::Receiver<AppState>| {
            let mut data_futures = Vec::new();

            let state_stream = WatchStream::new(state_channel);

            let (reduced_state_tx, reduced_state_rx) = worker.provide_input_stream();

            data_futures
                .push(Self::state_reducer(state_stream, state_tx, reduced_state_tx).boxed());
            data_futures.push(worker.work(reduced_state_rx));

            async move {
                futures::future::join_all(data_futures).await;
            }
            .boxed()
        };

        self.runners.push(Box::new(runner));
        self
    }

    pub fn add_worker<Consumed, II, Produced, E>(
        mut self,
        worker: impl worker::Worker<'f, Consumed, II, Produced, E> + Send + Sync + 'static,
    ) -> Self
    where
        Consumed: fmt::Debug + Send + Sync + 'f,
        II: fmt::Debug + Send + Sync + 'f,
        Produced: InjectedTo<AppState> + fmt::Debug + Send + Sync + 'f,
        E: fmt::Debug + Send + Sync + 'f,
        AppState: Reduced<Consumed>,
    {
        let state_update = self.state_tx.clone();
        let worker = Box::new(worker);
        let runner = |state_channel: watch::Receiver<AppState>| {
            let mut data_futures = Vec::new();
            let (inducer_tx, inducer_rx) = mpsc::channel::<Produced>(100);

            let state_stream = WatchStream::new(state_channel.clone());

            let (reduced_state_tx, reduced_state_rx) = worker.provide_input_stream();

            // `state_reducer and `state_mapper` propagate state to each consumer
            data_futures.push(
                Self::state_reducer(state_stream, state_update.clone(), reduced_state_tx).boxed(),
            );
            data_futures
                .push(Self::state_injector(inducer_rx, state_channel, state_update).boxed());
            data_futures.push(worker.work(reduced_state_rx, inducer_tx));

            async move {
                futures::future::join_all(data_futures).await;
            }
            .boxed()
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
