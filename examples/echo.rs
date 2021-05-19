use bilidanmu::*;
use futures::StreamExt;
use log::*;

const ROOM_ID: u64 = 6;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let client = bilidanmu::connection::new_client()?;

    let danmu_info = bilidanmu::requests::DanmuInfo::new(&client, ROOM_ID).await?;
    let server = &danmu_info.servers[0];
    let url = server.url();

    let mut connection =
        bilidanmu::connection::DanmuConnection::new(&url, ROOM_ID, danmu_info.token).await?;
    while let Some(msg) = connection.next().await {
        match msg {
            Ok(msg) => {
                info!("received msg: {:?}", msg);
            }
            Err(e) => {
                error!("error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}
