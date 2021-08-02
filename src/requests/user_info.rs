use crate::requests::prelude::*;

/// 用户信息，返回如粉丝数、头像等信息
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserInfo {
    // pub mid: u64,
    pub name: String,
    /// 头像链接
    #[serde(rename = "face")]
    pub avatar_url: String,

    /// 粉丝数
    #[serde(rename = "fans")]
    pub followers: u64,

    /// 关注数
    #[serde(rename = "attention")]
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

        #[derive(Debug, Deserialize)]
        struct Outer {
            card: UserInfo,
        }

        Box::pin(async move {
            let outer: Outer = r.await?.bili_data().await?;
            Ok(outer.card)
        })
    }
}

#[tokio::test]
async fn test_user_info() {
    let client = crate::connection::new_client().unwrap();
    let info = UserInfo::request(&client, 672328094).await.unwrap();
    assert_eq!(info.name, "嘉然今天吃什么");
}
