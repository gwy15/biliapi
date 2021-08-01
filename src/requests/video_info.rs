use chrono::{DateTime, Utc};
use serde_with::{serde_as, DurationSeconds};
use std::time::Duration;

use super::prelude::*;

/// 一个 BV 视频的信息
///
/// 从 `https://api.bilibili.com/x/web-interface/view?bvid={bv}` 获取
#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct VideoInfo {
    pub bvid: String,
    pub aid: u64,
    /// 稿件分P总数
    pub videos: usize,
    pub title: String,

    /// 稿件发布时间
    #[serde(rename = "pubdate", with = "chrono::serde::ts_seconds")]
    pub publish_at: DateTime<Utc>,

    /// 用户投稿时间
    #[serde(rename = "ctime", with = "chrono::serde::ts_seconds")]
    pub create_at: DateTime<Utc>,

    pub desc: String,

    /// 稿件总时长(所有分P)
    #[serde_as(as = "DurationSeconds<u64>")]
    pub duration: Duration,

    /// 视频封面 url
    #[serde(rename = "pic")]
    pub cover_url: String,

    pub stat: VideoStat,
}

/// 视频统计信息，点赞、弹幕数量等
#[derive(Debug, Deserialize, Serialize)]
pub struct VideoStat {
    pub aid: u64,
    pub view: u64,
    /// 弹幕数量
    pub danmaku: u64,
    pub reply: u64,
    pub favorite: u64,
    pub coin: u64,
    pub share: u64,
    pub now_rank: u64,
    /// 历史最高排行
    #[serde(rename = "his_rank")]
    pub history_rank: u64,

    pub like: u64,
    /// 点踩数量，恒为0，没有数据
    pub dislike: u64,
    /// 警告/争议提示信息
    pub argue_msg: String,
}

impl Request for VideoInfo {
    /// bv 号
    type Args = String;

    fn request(client: &Client, args: Self::Args) -> RequestResponse<Self> {
        const URL: &str = "https://api.bilibili.com/x/web-interface/view";
        let request = client.get(URL).query(&[("bvid", &args)]).send();
        Box::pin(async move { request.await?.bili_data().await })
    }
}

#[tokio::test]
async fn test_video_info() -> Result<()> {
    let client = crate::connection::new_client()?;
    let info = VideoInfo::request(&client, "BV1QB4y1u7Jj".to_string()).await?;
    assert_eq!(info.aid, 588385189);
    assert!(info.stat.like > 4100);
    assert!(info.title.contains("亲爱的那不是爱情"));

    Ok(())
}
