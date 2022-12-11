use core::fmt;

use app::{worker::ConsumerWorker, FutureExt, mpsc};
use serde::Deserialize;
use tg_api::Api;
use tokio::sync::watch;

pub mod implementation;


#[derive(Deserialize )]
pub struct TelegramReporterConfig {
    #[serde(rename="token", default="bot_token_from_env")]
    pub bot_token: String,
    pub storage_path: String,
}


fn bot_token_from_env() -> String {
    std::env::var("BOT_KEY").expect("Provide bot key via config or via env var 'BOT_KEY'")
}


pub struct TelegramReporter {
    api: Api,
    storage_path: String,
}

impl TelegramReporter {
    pub fn new(config: TelegramReporterConfig) -> Self {
        Self {
            api: Api::new(config.bot_token),
            storage_path: config.storage_path,
        }
    }
}



impl<'f, T> ConsumerWorker<'f, Vec<T>> for TelegramReporter 
where 
    T: fmt::Display + Sync + Send + 'f
{

    fn work(
            self: Box<Self>,
            state_rx: watch::Receiver<Option<Vec<T>>>,
        ) -> app::BoxFuture<'f, ()>

    {
        

        async move {
            let mut futures = Vec::new();
            let (tx, rx) = mpsc::channel(10);
            futures.push(self.bot_loop(tx).boxed());
            futures.push(self.send_loop(rx, state_rx).boxed());

            futures::future::join_all(futures).await;
        }.boxed()
    }
}
