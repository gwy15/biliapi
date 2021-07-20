use crate::requests::prelude::*;

/// 用户信息，返回如粉丝数、头像等信息
#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub mid: u64,
    pub name: String,
    /// 头像链接
    #[serde(rename = "face")]
    pub avatar_url: String,

    /// 粉丝数
    pub followers: u64,

    /// 关注数
    pub following: u64,

    /// 个性签名
    pub sign: String,
    // level_info
}

impl Request for UserInfo {
    /// mid
    type Args = u64;

    fn request(client: &Client, args: Self::Args) -> RequestResponse<Self> {
        const URL: &str = "https://api.bilibili.com/x/web-interface/card";
        let r = client.get(URL).query(&[("mid", args)]).send();
        Box::pin(async move { r.await?.bili_data().await })
    }
}
