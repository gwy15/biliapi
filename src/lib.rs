#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

pub mod connection;
pub mod requests;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Websocket error: {0}")]
    WebSocket(#[from] async_tungstenite::tungstenite::Error),

    #[error("Unexpected status code: {0}")]
    StatusCode(reqwest::StatusCode),

    #[error("Failed to parse http body as expected json.")]
    Serde(#[from] serde_json::Error),

    #[error("Bilibili error: ({}) {}", .code, .message)]
    BiliCustom { code: i64, message: String },

    #[error("The request seems ok but no data is found.")]
    DataNotFound,
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub use requests::Request;
