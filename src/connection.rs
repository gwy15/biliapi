//! 连接模块，包括 http client 和 websocket （直播间）连接
//!
#[cfg(feature = "live")]
use futures::{stream::SplitStream, FutureExt, Stream, StreamExt};
use reqwest::Client;
#[cfg(feature = "live")]
use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(feature = "live")]
type WebSocketStream = async_tungstenite::WebSocketStream<async_tungstenite::tokio::ConnectStream>;
#[cfg(feature = "live")]
use async_tungstenite::tungstenite::Error as WsError;

#[cfg(feature = "live")]
use crate::ws_protocol;
#[cfg(feature = "live")]
type WsResult<T> = Result<T, WsError>;

/// 创建一个新的 http 连接
pub fn new_client() -> reqwest::Result<Client> {
    const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    trace!("user agent name: {}", USER_AGENT);
    Client::builder()
        .user_agent(USER_AGENT)
        .cookie_store(true)
        .build()
}

#[cfg(feature = "live")]
/// 直播间 websocket 连接，实现了 [`Stream`][`futures::Stream`]
///
/// # Example
/// ```no_run
/// # use biliapi::connection::LiveConnection;
/// # let (url, room_id, token) = ("", 1, "".to_string());
/// # tokio_test::block_on(async {
/// use futures::StreamExt;
/// // 这些信息可以从 InfoByRoom 接口拿到
/// let mut con = LiveConnection::new(url, room_id, token).await.unwrap();
/// while let Some(msg) = con.next().await {
///     let msg = msg.unwrap();
/// }
/// # });
/// ```
pub struct LiveConnection {
    room_id: u64,
    heartbeat_future: Pin<Box<dyn Future<Output = WsResult<()>> + Send>>,
    read: SplitStream<WebSocketStream>,
    buffered_msg: VecDeque<ws_protocol::Packet>,
}
#[cfg(feature = "live")]
impl LiveConnection {
    /// 从 url 建立一个新连接，需要 room_id 和 token，这些数据可以从
    /// [`InfoByRoom`][`crate::requests::InfoByRoom`] 拿到
    pub async fn new(url: &str, room_id: u64, token: String) -> WsResult<Self> {
        let (websocket, _http) = async_tungstenite::tokio::connect_async(url).await?;
        let (write, read) = websocket.split();
        // start sending
        let heartbeat_future = Box::pin(async move {
            use futures::prelude::*;
            let mut write = write;
            write
                .send(ws_protocol::Packet::auth(room_id, &token).into())
                .await?;
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                debug!("sending heartbeat...");
                write
                    .send(ws_protocol::Packet::heartbeat().into())
                    .await
                    .map_err(|e| {
                        debug!("failed to send heartbeat: {:?}", e);
                        e
                    })?;
            }
        });
        Ok(Self {
            room_id,
            heartbeat_future,
            read,
            buffered_msg: VecDeque::new(),
        })
    }
}
#[cfg(feature = "live")]
impl Stream for LiveConnection {
    type Item = crate::Result<ws_protocol::Packet>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // try poll heartbeat first
        match self.heartbeat_future.poll_unpin(cx) {
            Poll::Ready(Err(e)) => {
                warn!("The heartbeat future exited unexpectedly: {:?}", e);
                return Poll::Ready(Some(Err(e.into())));
            }
            Poll::Ready(Ok(_)) => unreachable!(),
            Poll::Pending => {}
        }
        // try buffered messages
        if let Some(msg) = self.buffered_msg.pop_front() {
            return Poll::Ready(Some(Ok(msg)));
        }

        // now get a message
        match self.read.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(ws_message))) => {
                let msgs = ws_protocol::Packet::from_ws_message(ws_message, self.room_id)?;
                self.buffered_msg.extend(msgs);
                match self.buffered_msg.pop_front() {
                    Some(msg) => Poll::Ready(Some(Ok(msg))),
                    None => Poll::Pending,
                }
            }
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e.into()))),
            Poll::Ready(None) => Poll::Ready(None),
        }
    }
}
