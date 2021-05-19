use super::prelude::*;

/// 获取用于连接直播间的弹幕服务器
///
/// 从 /xlive/web-room/v1/index/getDanmuInfo 获取
#[derive(Debug, Deserialize)]
pub struct DanmuInfo {
    pub token: String,
    #[serde(rename = "host_list")]
    pub servers: Vec<DanmuServer>,
}

/// [`DanmuInfo`] 的子信息，连接直播间的弹幕服务器
#[derive(Debug, Deserialize)]
pub struct DanmuServer {
    pub host: String,
    pub port: u16,
    pub wss_port: u16,
    pub ws_port: u16,
}
impl DanmuServer {
    /// 获取对应的 url（wss 协议）
    pub fn url(&self) -> String {
        format!("wss://{}:{}/sub", self.host, self.wss_port)
    }
}

impl Request for DanmuInfo {
    type Args = u64;

    fn request(client: &Client, args: Self::Args) -> RequestResponse<Self> {
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
        let info = crate::requests::DanmuInfo::request(&client, 2).await?;
        assert!(info.servers.len() > 0);
        Ok(())
    }
}
