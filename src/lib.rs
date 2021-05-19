//! 哔哩哔哩部分 API 接口实现
//!
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

pub mod connection;
pub mod requests;
pub mod ws_protocol;

/// 各种可能遇到的错误
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// 在连接 http 的时候可能发生的错误
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// 在连接 websocket 的时候可能发生的错误
    #[error("Websocket error: {0}")]
    WebSocket(#[from] async_tungstenite::tungstenite::Error),

    /// 在连接 http 的时候可能返回非 200 的返回码（如被频控、url 不存在）
    #[error("Unexpected status code: {0}")]
    StatusCode(reqwest::StatusCode),

    /// 在解析的时候期望的数据类型和实际的不匹配无法解析
    #[error("Failed to parse http body as expected json.")]
    Serde(#[from] serde_json::Error),

    /// 哔哩哔哩定义在 http 应用层之上的一个应用层错误
    #[error("Bilibili error: ({}) {}", .code, .message)]
    BiliCustom { code: i64, message: String },

    /// 哔哩哔哩返回的结构中没有 data 字段
    #[error("The request seems ok but no data is found.")]
    DataNotFound,

    /// 解析 websocket 协议时发生的错误
    #[error("Failed to parse as bilibili protocol: {0}")]
    Protocol(#[from] ws_protocol::ParseError),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub use requests::Request;
