use serde::{de::DeserializeOwned, Serialize};
use telegram_bot_raw::{
    GetUpdates, MessageOrChannelPost, ResponseType, ResponseWrapper, SendMessage,
};
use tracing::error;
use update::Update;
use url::Url;

pub mod update;

#[derive(Clone)]
pub struct Api {
    token: String,
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

    async fn request<I, O>(&self, method: &str, input: I) -> O
    where
        I: Serialize,
        O: DeserializeOwned + Default,
    {
        let mut url = self.url(method);
        let query = serde_qs::to_string(&input).unwrap();
        url.set_query(Some(&query));

        let content = reqwest::get(url).await.unwrap().text().await.unwrap();
        let response: ResponseWrapper<O> = serde_json::from_str(&content).unwrap();
        match response {
            ResponseWrapper::Success { result } => result,
            ResponseWrapper::Error {
                description,
                parameters,
            } => {
                error!(?description, ?parameters, "Error during request");
                O::default()
            }
        }
    }

    pub async fn send_message<'s>(&self, request: SendMessage<'s>) -> Option<MessageOrChannelPost> {
        self.request("sendMessage", request).await
    }

    pub async fn get_updates(&self, request: GetUpdates) -> Vec<Update> {
        self.request("getUpdates", request).await
    }
}
