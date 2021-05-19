use bilidanmu::*;
use futures::StreamExt;
use log::*;

const ROOM_ID: u64 = 650;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let client = bilidanmu::connection::new_client()?;

    let room_info = bilidanmu::requests::InfoByRoom::new(&client, ROOM_ID).await?;
    let room_id = room_info.room_info.room_id;

    let danmu_info = bilidanmu::requests::DanmuInfo::new(&client, room_id).await?;
    let server = &danmu_info.servers[0];
    let url = server.url();

    let mut connection =
        bilidanmu::connection::DanmuConnection::new(&url, room_id, danmu_info.token).await?;
    while let Some(msg) = connection.next().await {
        match msg {
            Ok(msg) => {
                info!("{}\n{}", msg.operation, msg.body);
            }
            Err(e) => {
                error!("error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}
