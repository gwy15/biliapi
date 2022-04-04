//! 哔哩哔哩部分 API 接口实现
//!
//! # features
//! 默认不指定 TLS 后端，需要手动指定。
//!
//! ## native-tls
//! 使用 native tls
//!
//! ## rustls
//! 使用 rustls
//!
//! # live
//! 启用 b 站直播相关 api，默认关闭
//!
//! # live-native-tls / live-rustls
//! 设置 live 的 tls 后端。这个 feature 需要跟 native-tls / rustls 同时设置，因为目前 cargo 的
//! [weak dependency features](https://doc.rust-lang.org/cargo/reference/unstable.html#weak-dependency-features)
//! 还没 stable

#[cfg(not(any(feature = "rustls", feature = "native-tls")))]
compile_error!("至少应该启用一个 rustls 或是 native-tls features");
#[cfg(all(feature = "rustls", feature = "native-tls"))]
compile_error!("rustls 和 native-tls features 只能同时启用一个");
#[cfg(all(
    feature = "live",
    not(any(feature = "live-native-tls", feature = "live-rustls"))
))]
compile_error!("live 必须和 live-native-tls 或 live-rustls 之中的一个同时启用");

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

pub mod connection;
pub mod requests;
#[cfg(feature = "live")]
pub mod ws_protocol;

/// 各种可能遇到的错误
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// 在连接 http 的时候可能发生的错误
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[cfg(feature = "live")]
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

    #[cfg(feature = "live")]
    /// 解析 websocket 协议时发生的错误
    #[error("Failed to parse as bilibili protocol: {0}")]
    Protocol(#[from] ws_protocol::ParseError),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub use requests::Request;
