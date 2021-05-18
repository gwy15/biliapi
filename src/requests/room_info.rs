use super::prelude::*;

/// 从 /xlive/web-room/v1/index/getInfoByRoom 接口拿到的信息
#[derive(Debug, Deserialize)]
pub struct InfoByRoom {
    pub room_info: RoomInfo,
}
#[derive(Debug, Deserialize)]
pub struct RoomInfo {
    pub room_id: u64,
    pub short_id: u64,
    pub cover: String,
    pub keyframe: String,
}

impl Request for InfoByRoom {
    type Args = i64;
    type Future = Pin<Box<dyn Future<Output = Result<Self>>>>;

    fn new(client: &Client, args: Self::Args) -> Self::Future {
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
        let info = crate::requests::InfoByRoom::new(&client, 646).await?;
        assert_eq!(info.room_info.room_id, 21133);
        assert_eq!(info.room_info.short_id, 646);

        let info = crate::requests::InfoByRoom::new(&client, 21133).await?;
        assert_eq!(info.room_info.room_id, 21133);
        assert_eq!(info.room_info.short_id, 646);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_non_exist_room_info() -> Result<()> {
        let client = crate::connection::new_client()?;
        let info = crate::requests::InfoByRoom::new(&client, 38)
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
