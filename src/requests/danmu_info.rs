use super::prelude::*;

/// 获取直播的弹幕服务器
///
/// 从 /xlive/web-room/v1/index/getDanmuInfo 获取
#[derive(Debug, Deserialize)]
pub struct DanmuInfo {
    pub token: String,
    #[serde(rename = "host_list")]
    pub servers: Vec<DanmuServer>,
}

#[derive(Debug, Deserialize)]
pub struct DanmuServer {
    pub host: String,
    pub port: u16,
    pub wss_port: u16,
    pub ws_port: u16,
}

impl Request for DanmuInfo {
    type Args = i64;

    fn new(client: &Client, args: Self::Args) -> RequestResponse<Self> {
        const DANMU_INFO_URL: &str =
            "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo";

        let request = client.get(DANMU_INFO_URL).query(&[("id", args)]).send();
        Box::pin(async move { request.await?.bili_data().await })
    }
}

#[cfg(test)]
mod tests {
    use crate::Request;
    use anyhow::*;
    #[tokio::test]
    async fn test_get_danmu_info() -> Result<()> {
        let client = crate::connection::new_client()?;
        let info = crate::requests::DanmuInfo::new(&client, 2).await?;
        assert!(info.servers.len() > 0);
        Ok(())
    }
}
