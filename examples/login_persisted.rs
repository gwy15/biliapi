//! 这个实例进行登录并查询一个需要登录的接口
use std::{path::Path, sync::Arc, time::Duration};

use anyhow::*;
use biliapi::Request;
use cookie_store::CookieStore;
use log::*;
use qrcode::{render::unicode, QrCode};
use reqwest::Client;
use reqwest_cookie_store::CookieStoreMutex;
use tokio::{fs, time::sleep};

async fn load_persisted_cookie_store_from_file(f: &Path) -> Result<CookieStore> {
    let cookies = fs::read_to_string(f).await?;
    let store = CookieStore::load_json(cookies.as_bytes()).map_err(|e| anyhow!(e))?;
    Ok(store)
}

async fn persisted_client() -> Result<(Client, Arc<CookieStoreMutex>)> {
    let cookies = load_persisted_cookie_store_from_file("./persisted_cookies.json".as_ref())
        .await
        .unwrap_or_else(|e| {
            warn!("failed to load cookies: {:?}", e);
            CookieStore::default()
        });

    let cookies = Arc::new(CookieStoreMutex::new(cookies));

    let client = reqwest::ClientBuilder::new()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.164 Safari/537.36")
        .cookie_provider(Arc::clone(&cookies))
        .build()?;

    Ok((client, cookies))
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;
    pretty_env_logger::init();

    let (client, cookies) = persisted_client().await?;

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
    let mut retry: i32 = 0;
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

    {
        let mut f = std::fs::File::create("persisted_cookies.json")?;
        let c = cookies.lock().unwrap();
        c.save_json(&mut f).map_err(|e| anyhow!(e))?;
    }

    Ok(())
}
