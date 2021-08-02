//! 基于 HTTP 的各种请求
//!
//! # Example
//! ```
//! use biliapi::Request;
//! # tokio_test::block_on(async {
//! let client = biliapi::connection::new_client().unwrap();
//! let info = biliapi::requests::InfoByRoom::request(&client, 1).await.unwrap();
//! // 拿到长房号
//! assert_eq!(info.room_info.room_id, 5440);
//! # });
//! ```
//!
mod prelude {
    pub use reqwest::{Client, Response, StatusCode};
    pub use serde::de::DeserializeOwned;
    pub use std::{future::Future, pin::Pin};

    pub use crate::{Error, Result};

    pub(super) use super::BiliResponseExt;
    pub use super::{Request, RequestResponse};
}

use prelude::*;

/// 哔哩哔哩返回的 http 原始 response 对应的结构
#[derive(Debug, Deserialize, Serialize, Clone)]
struct BiliResponse<T> {
    code: i64,

    #[serde(default)]
    message: String,

    // 不知道干啥的
    // ttl: i64
    #[serde(bound(deserialize = "T: serde::Deserialize<'de>"))]
    #[serde(default = "Option::default")]
    data: Option<T>,
}
impl<T: DeserializeOwned> BiliResponse<T> {
    #[allow(unused)]
    pub fn into_data(self) -> Option<T> {
        self.data
    }
    pub async fn from_response(response: Response) -> Result<T> {
        if response.status() != StatusCode::OK {
            return Err(Error::StatusCode(response.status()));
        }
        let response_text = response.text().await?;
        let this: Self = serde_json::from_str(&response_text).map_err(|e| {
            debug!("response text = {}", response_text);
            e
        })?;
        if this.code != 0 {
            return Err(Error::BiliCustom {
                code: this.code,
                message: this.message,
            });
        }
        match this.data {
            Some(data) => Ok(data),
            None => Err(Error::DataNotFound),
        }
    }
}

/// 这个 trait 允许直接对 [`Response`] 调用 `bili_data().await`
///
/// ```no_run
/// use biliapi::requests::BiliResponseExt;
///
/// # async fn dox() -> Result<(), Box<dyn std::error::Error>> {
/// let response: reqwest::Response = reqwest::Client::new()
///     .get("https://example.com")
///     .send().await?;
///
/// let data: i32 = response.bili_data().await?;
/// # Ok(()) }
/// ```
///
pub trait BiliResponseExt<T: DeserializeOwned> {
    fn bili_data(self) -> Pin<Box<dyn Future<Output = Result<T>> + Send>>;
}
impl<T: DeserializeOwned> BiliResponseExt<T> for Response {
    fn bili_data(self) -> Pin<Box<dyn Future<Output = Result<T>> + Send>> {
        Box::pin(async move { BiliResponse::<T>::from_response(self).await })
    }
}

/// API 接口的实现 trait
///
/// 所有对 bilibili 的请求都应该实现这个 trait，如
/// ```no_run
/// use biliapi::requests::{Request, BiliResponseExt, RequestResponse};
/// use serde::Deserialize;
/// use reqwest::Client;
///
/// #[derive(Debug, Deserialize)]
/// struct SomeApi {
///     pub field: i32
/// }
/// impl Request for SomeApi {
///     type Args = i64;
///     fn request(client: &Client, args: i64) -> RequestResponse<Self> {
///         let request = client.get("https://api.bilibili.com/some/api")
///             .query(&[("id", args)])
///             .send();
///         Box::pin(async move {
///             // 这里需要引入 `BiliResponseExt`
///             request.await?.bili_data().await
///         })
///
///     }
/// }
/// ```
pub trait Request: DeserializeOwned {
    /// 请求对应的参数
    type Args;

    /// 请求的实现
    fn request(client: &Client, args: Self::Args) -> RequestResponse<Self>;
}
/// [`Request`] trait 返回结果的封装，本质就是 `Pin<Box<dyn Future<Output = Result<T>>>>`
pub type RequestResponse<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

mod room_info;
pub use room_info::{InfoByRoom, RoomInfo};

mod danmu_info;
pub use danmu_info::{DanmuInfo, DanmuServer};

mod video_info;
pub use video_info::{VideoInfo, VideoStat};

mod login;
pub use login::{CheckQrLogin, QrLoginRequest};

mod uploader_stat;
pub use uploader_stat::UploaderStat;

mod user_info;
pub use user_info::UserInfo;

mod vote_info;
pub use vote_info::VoteInfo;

mod my_account_info;
pub use my_account_info::MyAccountInfo;
