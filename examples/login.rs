//! 这个实例进行登录并查询一个需要登录的接口
use std::time::Duration;

use anyhow::*;
use biliapi::Request;
use log::*;
use qrcode::{render::unicode, QrCode};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;
    pretty_env_logger::init();

    let client = biliapi::connection::new_client()?;
    let request = biliapi::requests::QrLoginRequest::request(&client, ()).await?;

    let code = QrCode::new(request.url.as_bytes())?;
    let string = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    info!("请使用哔哩哔哩手机客户端扫描下面的二维码以登录");
    println!("{}", string);

    // 先等5秒
    sleep(Duration::from_secs(5)).await;
    // 每隔3秒轮询一次，最多轮询10次
    let mut retry = 0;
    loop {
        sleep(Duration::from_secs(3)).await;
        let resp =
            biliapi::requests::CheckQrLogin::request(&client, request.oauth_key.clone()).await?;
        match resp.is_success() {
            Some(true) => {
                info!("登录成功！");
                break;
            }
            Some(false) => {
                bail!("登录失败");
            }
            None => {
                if retry > 10 {
                    bail!("登录超时了");
                }
            }
        }
        retry += 1;
    }

    // 查询一个只有登录状态能查询的 API
    let bird_info = biliapi::requests::UploaderStat::request(&client, 282994).await?;
    info!("获取信息成功：{:?}", bird_info);
    Ok(())
}
