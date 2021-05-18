mod prelude {
    pub use reqwest::{Client, Response, StatusCode};
    pub use serde::de::DeserializeOwned;
    pub use std::{future::Future, pin::Pin};

    pub use crate::{Error, Result};

    pub use super::{BiliResponse, BiliResponseExt, Request, RequestResponse};
}

use prelude::*;

#[derive(Debug, Deserialize)]
pub struct BiliResponse<T> {
    code: i64,

    message: String,

    // 不知道干啥的
    // ttl: i64
    #[serde(bound(deserialize = "T: serde::Deserialize<'de>"))]
    #[serde(default = "Option::default")]
    data: Option<T>,
}
impl<T: DeserializeOwned> BiliResponse<T> {
    pub fn into_data(self) -> Option<T> {
        self.data
    }
    pub async fn from_response(response: Response) -> Result<T> {
        if response.status() != StatusCode::OK {
            return Err(Error::StatusCode(response.status()));
        }
        let response_text = response.text().await?;
        let this: Self = serde_json::from_str(&response_text)?;
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
pub trait BiliResponseExt<T: DeserializeOwned> {
    fn bili_data(self) -> Pin<Box<dyn Future<Output = Result<T>>>>;
}
impl<T: DeserializeOwned> BiliResponseExt<T> for Response {
    fn bili_data(self) -> Pin<Box<dyn Future<Output = Result<T>>>> {
        Box::pin(async move { BiliResponse::<T>::from_response(self).await })
    }
}

/// 所有对 bilibili 的请求都应该实现这个 trait
pub trait Request: DeserializeOwned {
    type Args;

    fn new(client: &Client, args: Self::Args) -> RequestResponse<Self>;
}
pub type RequestResponse<T> = Pin<Box<dyn Future<Output = Result<T>>>>;

mod room_info;
pub use room_info::{InfoByRoom, RoomInfo};

mod danmu_info;
pub use danmu_info::{DanmuInfo, DanmuServer};
