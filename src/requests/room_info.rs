use super::prelude::*;

/// 通过房号拿到直播间的信息
///
/// API 源：/xlive/web-room/v1/index/getInfoByRoom
#[derive(Debug, Deserialize)]
pub struct InfoByRoom {
    /// 直播间信息
    pub room_info: RoomInfo,
}
/// [`InfoByRoom`] 的子信息，代表直播间信息
#[derive(Debug, Deserialize)]
pub struct RoomInfo {
    /// 长房号，如 5440
    pub room_id: u64,
    /// 短房号，如 1
    pub short_id: u64,
    /// 直播间封面 url
    pub cover: String,
    /// 直播间关键帧
    pub keyframe: String,
}

impl Request for InfoByRoom {
    type Args = u64;

    fn request(client: &Client, args: Self::Args) -> RequestResponse<Self> {
        const ROOM_INIT_URL: &str =
            "https://api.live.bilibili.com/xlive/web-room/v1/index/getInfoByRoom";

        let request = client.get(ROOM_INIT_URL).query(&[("room_id", args)]).send();

        Box::pin(async { request.await?.bili_data().await })
    }
}

#[cfg(test)]
mod tests {
    use crate::Request;
    use anyhow::*;

    #[tokio::test]
    async fn test_get_cgg_room_info() -> Result<()> {
        let client = crate::connection::new_client()?;
        // 超果果
        let info = crate::requests::InfoByRoom::request(&client, 646).await?;
        assert_eq!(info.room_info.room_id, 21133);
        assert_eq!(info.room_info.short_id, 646);

        let info = crate::requests::InfoByRoom::request(&client, 21133).await?;
        assert_eq!(info.room_info.room_id, 21133);
        assert_eq!(info.room_info.short_id, 646);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_non_exist_room_info() -> Result<()> {
        let client = crate::connection::new_client()?;
        let info = crate::requests::InfoByRoom::request(&client, 38)
            .await
            .err()
            .unwrap();
        assert_eq!(
            info.to_string(),
            "Bilibili error: (19002000) 获取初始化数据失败"
        );
        Ok(())
    }
}
