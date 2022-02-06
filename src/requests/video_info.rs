use chrono::{DateTime, Utc};
use serde_with::{serde_as, DurationSeconds};
use std::time::Duration;

use super::prelude::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VideoOwner {
    pub mid: u64,
    pub name: String,
    #[serde(rename = "face")]
    pub avatar_url: String,
}

/// 一个 BV 视频的信息
///
/// 从 `https://api.bilibili.com/x/web-interface/view?bvid={bv}` 获取
#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VideoInfo {
    pub bvid: String,
    pub aid: u64,
    /// 稿件分P总数
    pub videos: usize,
    pub title: String,
    /// 1：原创 2：转载
    #[serde(default)]
    pub copyright: i32,

    /// 稿件发布时间
    #[serde(rename = "pubdate", with = "chrono::serde::ts_seconds")]
    pub publish_at: DateTime<Utc>,

    /// 用户投稿时间
    #[serde(rename = "ctime", with = "chrono::serde::ts_seconds_option", default)]
    pub create_at: Option<DateTime<Utc>>,

    pub desc: String,

    /// 稿件总时长(所有分P)
    #[serde_as(as = "DurationSeconds<u64>")]
    pub duration: Duration,

    /// 视频封面 url
    #[serde(rename = "pic")]
    pub cover_url: String,

    pub stat: VideoStat,

    pub owner: VideoOwner,
}

/// 视频统计信息，点赞、弹幕数量等
#[derive(Debug, Deserialize, Serialize, Clone)]
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
    /// 警告/争议提示信息，有的时候没有
    #[serde(default)]
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

#[cfg(test)]
#[tokio::test]
async fn test_video_info() -> Result<()> {
    let client = crate::connection::new_client()?;
    let info = VideoInfo::request(&client, "BV1QB4y1u7Jj".to_string()).await?;
    assert_eq!(info.aid, 588385189);
    assert!(info.stat.like > 4100);
    assert!(info.title.contains("亲爱的那不是爱情"));

    Ok(())
}

#[test]
fn test_video_info_parse_from_tag() {
    let s = r#"
    {
        "aid": 759539804,
        "videos": 1,
        "tid": 31,
        "tname": "翻唱",
        "copyright": 2,
        "pic": "http://i1.hdslb.com/bfs/archive/c951667bccc7dd4ddbe425f83888e8a5553ce11e.jpg",
        "title": "A-SOUL/珈乐】《历历万乡》",
        "pubdate": 1628088921,
        "desc": "https://live.bilibili.com/22634198/\nhttps://live.bilibili.com/22634198/\n珈乐 2021-8-4 直播剪辑\n\n\n\n点个关注呗~\nA-SOUL主页链接：https://space.bilibili.com/703007996/ \n向晚：https://space.bilibili.com/672346917\n乃琳：https://space.bilibili.com/672342685\n珈乐：https://space.bilibili.",
        "state": 0,
        "duration": 216,
        "rights": {
            "bp": 0,
            "elec": 0,
            "download": 0,
            "movie": 0,
            "pay": 0,
            "hd5": 0,
            "no_reprint": 0,
            "autoplay": 1,
            "ugc_pay": 0,
            "is_cooperation": 0,
            "ugc_pay_preview": 0,
            "no_background": 0
        },
        "owner": {
            "mid": 19409621,
            "name": "Bahumt",
            "face": "http://i2.hdslb.com/bfs/face/19d6aeaf72e4914c46aa29fbfd90906aa656f378.jpg"
        },
        "stat": {
            "aid": 759539804,
            "view": 1,
            "danmaku": 0,
            "reply": 0,
            "favorite": 0,
            "coin": 0,
            "share": 0,
            "now_rank": 0,
            "his_rank": 0,
            "like": 0,
            "dislike": 0
        },
        "dynamic": "",
        "cid": 382633251,
        "dimension": {
            "width": 1920,
            "height": 1080,
            "rotate": 0
        },
        "short_link_v2": "https://b23.tv/BV1b64y1B79o",
        "bvid": "BV1b64y1B79o"
    }"#;
    serde_json::from_str::<VideoInfo>(s).unwrap();
}
