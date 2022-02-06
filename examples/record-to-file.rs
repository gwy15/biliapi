use anyhow::Result;
use biliapi::Request;
use futures::StreamExt;
use clap::Parser;
use log::*;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio::{fs, io::AsyncWriteExt};

#[derive(Debug, Parser)]
struct Opts {
    #[clap(help = "The live room id")]
    room_id: u64,

    #[clap(
        long,
        short,
        help = "The output file",
        default_value = "recorded.json"
    )]
    output: PathBuf,
}

async fn run<F: tokio::io::AsyncWrite + Unpin>(
    room_id: u64,
    f: &mut F,
    client: &reqwest::Client,
) -> Result<()> {
    // 拿到弹幕数据
    let danmu_info = biliapi::requests::DanmuInfo::request(&client, room_id).await?;
    let server = &danmu_info.servers[0];
    let url = server.url();

    let mut connection =
        biliapi::connection::LiveConnection::new(&url, room_id, danmu_info.token).await?;
    info!("room {} connected.", room_id);
    let mut count: u64 = 0;
    while let Some(msg) = connection.next().await {
        match msg {
            Ok(msg) => {
                count += 1;
                f.write_all(
                    format!(
                        "{} {}\n{}\n",
                        chrono::Local::now().to_rfc3339(),
                        msg.operation,
                        msg.body
                    )
                    .as_bytes(),
                )
                .await?;
                if count > 0 && count % 1_000 == 0 {
                    info!("{} records written.", count);
                }
            }
            Err(e) => {
                error!("error: {:?}", e);
                return Err(e.into());
            }
        }
    }
    anyhow::bail!("Connection ran out.")
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    let opts = Opts::parse();

    let client = reqwest::ClientBuilder::new()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.0.4606.71 Safari/537.36")
        .build()?;

    info!("获取房间信息 {}", opts.room_id);
    let room_info = biliapi::requests::InfoByRoom::request(&client, opts.room_id).await?;
    let room_id = room_info.room_info.room_id;

    // 创建文件
    info!("recoding records to file {:?}", opts.output);
    let f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&opts.output)
        .await?;
    let mut f = tokio::io::BufWriter::new(f);

    let mut last_time = Instant::now();
    let mut err_counter = 0;
    static ALLOW_FAIL_DURATION: Duration = Duration::from_secs(5 * 60);
    loop {
        match run(room_id, &mut f, &client).await {
            Ok(_) => unreachable!(),
            Err(e) => {
                warn!("发生错误：{:?}", e);
                if Instant::now().duration_since(last_time) > ALLOW_FAIL_DURATION {
                    err_counter += 1;
                    if err_counter > 5 {
                        return Err(e);
                    }
                } else {
                    info!(
                        "距离上次失败已经过去了 {:?}",
                        Instant::now().duration_since(last_time)
                    );
                    err_counter = 1;
                }
                last_time = Instant::now();
            }
        }
    }
}
