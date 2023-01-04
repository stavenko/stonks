use serde::{de::DeserializeOwned, Serialize};
use telegram_bot_raw::{
    GetUpdates, MessageOrChannelPost, ResponseType, ResponseWrapper, SendMessage, ResponseParameters,
};
use tracing::error;
use update::Update;
use url::Url;

pub mod update;

#[derive(Clone)]
pub struct Api {
    token: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Serde-json {0}")]
    SerdeJson(serde_json::Error),
    #[error("Serde-qs {0}")]
    SerdeQs(serde_qs::Error),
    #[error("request {0}")]
    Request(reqwest::Error),
    #[error("tg error {description}")]
    Telegram{description:String, parameters: Option<ResponseParameters>}
}

impl Api {
    pub fn new(token: String) -> Self {
        Self { token }
    }

    fn url(&self, method: &str) -> Url {
        let mut url = Url::parse("https://api.telegram.org/").unwrap();
        url.set_path(&format!("bot{}/{}", self.token, method));
        url
    }

    async fn request<I, O>(&self, method: &str, input: I) -> Result<O, Error>
    where
        I: Serialize,
        O: DeserializeOwned
    {
        let mut url = self.url(method);
        let query = serde_qs::to_string(&input).unwrap();
        url.set_query(Some(&query));

        let content = reqwest::get(url).await?.text().await?;
        let response: ResponseWrapper<O> = serde_json::from_str(&content).unwrap();
        match response {
            ResponseWrapper::Success { result } => Ok(result),
            ResponseWrapper::Error {
                description,
                parameters,
            } => {
                Err(Error::Telegram { description, parameters })
            }
        }
    }

    pub async fn send_message<'s>(
        &self,
        request: SendMessage<'s>,
    ) -> Result<MessageOrChannelPost, Error> {
        self.request("sendMessage", request).await
    }

    pub async fn get_updates(&self, request: GetUpdates) -> Result<Vec<Update>, Error> {
        self.request("getUpdates", request).await
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::SerdeJson(value)
    }
}

impl From<serde_qs::Error> for Error {
    fn from(value: serde_qs::Error) -> Self {
        Error::SerdeQs(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Request(value)
    }
}
