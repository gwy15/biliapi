use async_tungstenite::tungstenite::Message as WsMessage;
use futures::{stream::SplitStream, FutureExt, Stream, StreamExt};
use reqwest::Client;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

type WebSocketStream = async_tungstenite::WebSocketStream<async_tungstenite::tokio::ConnectStream>;
use async_tungstenite::tungstenite::Error as WsError;

use crate::ws_protocol;
type WsResult<T> = Result<T, WsError>;

pub fn new_client() -> reqwest::Result<Client> {
    const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    trace!("user agent name: {}", USER_AGENT);
    Client::builder().user_agent(USER_AGENT).build()
}

pub struct DanmuConnection {
    heartbeat_future: Pin<Box<dyn Future<Output = WsError>>>,
    read: SplitStream<WebSocketStream>,
}
impl DanmuConnection {
    fn auth_packet(room_id: u64, token: &str) -> Vec<u8> {
        let payload = serde_json::json!({
            "uid": 0,
            "roomid": room_id,
            "protover": 2,
            "platform": "web",
            "clientver": "1.14.3",
            "type": 2,
            "key": token
        });
        ws_protocol::Operations::Auth.make(&payload)
    }

    pub async fn new(url: &str, room_id: u64, token: String) -> WsResult<Self> {
        let (websocket, _http) = async_tungstenite::tokio::connect_async(url).await?;
        let (write, read) = websocket.split();
        // start sending
        let heartbeat_future = Box::pin(async move {
            use futures::prelude::*;
            let mut write = write;
            if let Err(e) = write
                .send(WsMessage::Binary(Self::auth_packet(room_id, &token)))
                .await
            {
                return e;
            }
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                debug!("sending heartbeat...");
                // TODO: heatbeat
            }
        });
        Ok(Self {
            heartbeat_future,
            read,
        })
    }
}
impl Stream for DanmuConnection {
    type Item = WsResult<WsMessage>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // try poll heartbeat first
        match self.heartbeat_future.poll_unpin(cx) {
            Poll::Ready(e) => {
                warn!("The heartbeat future exited unexpectedly: {:?}", e);
                return Poll::Ready(Some(Err(e)));
            }
            Poll::Pending => {}
        }
        // now get a message
        self.read.poll_next_unpin(cx)
    }
}
