use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cannot parse message <{1}>: {0}")]
    SerdeError(serde_json::Error, String),

    #[error("Cannot get message: {0}")]
    TungsteniteError(tokio_tungstenite::tungstenite::Error),

    #[error("Cannot convert message to market feed message")]
    NotAMarketMessage,
}

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        Error::TungsteniteError(e)
    }
}
