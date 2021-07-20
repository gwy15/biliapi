use crate::requests::prelude::*;
use serde::Deserialize;

/// up 主的统计信息，需要登录才可以获取
#[derive(Debug, PartialEq)]
pub struct UploaderStat {
    /// 视频阅读数
    video_views: u64,

    /// 文章阅读数
    article_views: u64,

    /// 获赞数
    likes: u64,
}

impl<'de> Deserialize<'de> for UploaderStat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        pub struct Sub {
            view: u64,
        }
        #[derive(Deserialize)]
        pub struct Result {
            archive: Sub,
            article: Sub,
            likes: u64,
        }
        let r = Result::deserialize(deserializer)?;
        Ok(Self {
            video_views: r.archive.view,
            article_views: r.article.view,
            likes: r.likes,
        })
    }
}

impl Request for UploaderStat {
    /// 用户的 mid
    type Args = i64;
    fn request(client: &Client, args: Self::Args) -> RequestResponse<Self> {
        const URL: &str = "https://api.bilibili.com/x/space/upstat";
        let r = client.get(URL).query(&[("mid", args)]).send();
        Box::pin(async move { r.await?.bili_data().await })
    }
}

#[test]
fn test_uploader_stat_deser() {
    let s = r#"{
        "archive": {
            "view": 21498000
        },
        "article": {
            "view": 108263
        },
        "likes": 7733129
    }"#;
    let s: UploaderStat = serde_json::from_str(s).unwrap();
    assert_eq!(
        s,
        UploaderStat {
            video_views: 21498000,
            article_views: 108263,
            likes: 7733129
        }
    );
}
