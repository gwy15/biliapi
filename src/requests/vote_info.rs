//! 获取专栏投票信息
use crate::requests::prelude::*;
use chrono::{DateTime, Utc};

/// 通过 vote id 获取专栏投票信息，需要登录才可以获取具体票数
#[derive(Debug, Deserialize, Serialize)]
pub struct VoteInfo {
    pub vote_id: u64,

    pub title: String,

    pub choice_cnt: u64,
    /// 大概是总票数
    pub cnt: u64,

    #[serde(with = "chrono::serde::ts_seconds", rename = "endtime")]
    pub end_time: DateTime<Utc>,

    pub options: Vec<VoteOption>,

    #[serde(with = "chrono::serde::ts_seconds", rename = "starttime")]
    pub start_time: DateTime<Utc>,
}

impl Request for VoteInfo {
    // vote id
    type Args = u64;
    fn request(client: &Client, args: Self::Args) -> RequestResponse<Self> {
        let r = client
            .get("https://api.vc.bilibili.com/vote_svr/v1/vote_svr/vote_info")
            .query(&[("vote_id", args)])
            .send();

        #[derive(Debug, Deserialize)]
        struct Helper {
            pub info: VoteInfo,
        }

        Box::pin(async move {
            let helper: Helper = r.await?.bili_data().await?;
            Ok(helper.info)
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VoteOption {
    pub btn_str: String,
    pub cnt: u64,
    pub desc: String,
    pub idx: u32,
    pub title: String,
}
