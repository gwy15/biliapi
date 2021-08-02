//! 我的账号信息，可以用来判断当前 client 是否登录
use crate::requests::prelude::*;

/// 我的账号信息，需要鉴权
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MyAccountInfo {
    /// uid
    #[serde(rename = "mid")]
    uid: u64,

    #[serde(rename = "uname")]
    username: String,

    /// 签名
    sign: String,
}

impl Request for MyAccountInfo {
    type Args = ();

    fn request(client: &Client, _: Self::Args) -> RequestResponse<Self> {
        const URL: &str = "https://api.bilibili.com/x/member/web/account";
        let r = client.get(URL).send();
        Box::pin(async move { r.await?.bili_data().await })
    }
}
