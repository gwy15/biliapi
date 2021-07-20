//! 跟登录有关的请求，只实现了扫码登录
//!
//! 登录流程：先请求一个二维码，然后提示给用户，然后发起 [`CheckQrLogin`] 的轮询

use crate::requests::prelude::*;

/// 发起一次二维码登录请求
#[derive(Debug, Deserialize)]
pub struct QrLoginRequest {
    /// 二维码内容 url
    pub url: String,

    /// 扫码登录秘钥
    #[serde(rename = "oauthKey")]
    pub oauth_key: String,
}

impl Request for QrLoginRequest {
    type Args = ();
    fn request(client: &Client, _args: ()) -> RequestResponse<Self> {
        const URL: &str = "https://passport.bilibili.com/qrcode/getLoginUrl";
        let request = client.get(URL).send();

        Box::pin(async move { request.await?.bili_data().await })
    }
}

/// 检查二维码登录结果，需要轮询
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CheckQrLogin {
    /// -1：密钥错误
    /// -2：密钥超时
    /// -4：未扫描
    /// -5：未确认
    Code(i32),
    Success {
        url: String,
    },
}

impl CheckQrLogin {
    pub fn is_success(&self) -> Option<bool> {
        match self {
            // 错误或者超时，不可重试
            CheckQrLogin::Code(-1 | -2) => Some(false),
            // 可以重试
            CheckQrLogin::Code(-4 | -5) => None,
            CheckQrLogin::Code(code) => {
                warn!("Unknown check qr login code = {}", code);
                Some(false)
            }
            Self::Success { .. } => Some(true),
        }
    }
}

impl Request for CheckQrLogin {
    // 扫码登录秘钥
    type Args = String;

    fn request(client: &Client, oauth_key: String) -> RequestResponse<Self> {
        const URL: &str = "https://passport.bilibili.com/qrcode/getLoginInfo";
        let request = client.post(URL).form(&[("oauthKey", oauth_key)]).send();

        #[derive(Debug, Deserialize)]
        struct Response {
            status: bool,
            data: CheckQrLogin,
            #[serde(default)]
            message: String,
        }

        Box::pin(async move {
            let response: Response = request.await?.json().await?;
            Ok(response.data)
        })
    }
}

#[tokio::test]
async fn test_get_qr_login_request() -> Result<()> {
    let client = crate::connection::new_client()?;
    let r = QrLoginRequest::request(&client, ()).await?;
    dbg!(r);
    Ok(())
}
