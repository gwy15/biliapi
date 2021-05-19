//! bilibili 直播间传回来的数据

use std::fmt::{self, Debug, Display};

use async_tungstenite::tungstenite::Message as WsMessage;

/// 解析直播间数据时发生的错误
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Websocket packet type not supported: {0}")]
    WsTypeNotSupported(String),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Encoding error: {0}")]
    Encoding(#[from] std::string::FromUtf8Error),
}

/// pure magic
pub mod magic {

    /// I    |      H    |  H  |  I  |   I
    /// u32  |     u16   | u16 | u32 |  u32
    /// size | head size | ver |  op | seq_id
    pub const HEADER_SIZE: usize = 4 + 2 + 2 + 4 + 4;

    pub const VER_ZLIB_COMPRESSED: u16 = 2;
    pub const VER_NORMAL: u16 = 1;

    /// 已知的操作
    ///
    /// 从泄露代码的 app/service/main/broadcast/model/operation.go 可以看到命名
    #[enum_repr::EnumRepr(type = "u32")]
    #[derive(Debug, Clone, Copy)]
    pub enum KnownOperation {
        Handshake = 0,
        HandshakeReply = 1,
        Heartbeat = 2,
        HeartbeatReply = 3,
        SendMsg = 4,
        SendMsgReply = 5,
        DisconnectReply = 6,
        Auth = 7,
        AuthReply = 8,
        Raw = 9,
        ProtoReady = 10,
        ProtoFinish = 11,
        ChangeRoom = 12,
        ChangeRoomReply = 13,
        Register = 14,
        RegisterReply = 15,
        Unregister = 16,
        UnregisterReply = 17,
    }
}

/// 每个 packet 对应的 operation
#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Known(magic::KnownOperation),
    Unknown(u32),
}
impl Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::Known(op) => f.write_fmt(format_args!("{:?}", op)),
            Operation::Unknown(op) => f.write_fmt(format_args!("Unknown({})", op)),
        }
    }
}
impl From<Operation> for u32 {
    fn from(op: Operation) -> u32 {
        match op {
            Operation::Known(k) => k as u32,
            Operation::Unknown(u) => u,
        }
    }
}
impl From<u32> for Operation {
    fn from(u: u32) -> Operation {
        match magic::KnownOperation::from_repr(u) {
            Some(k) => Self::Known(k),
            None => Self::Unknown(u),
        }
    }
}

/// 对应 websocket 返回的 packet，基本上没进行处理
#[derive(Debug)]
pub struct Message {
    /// packet 对应的 operation，大部分应该都是 [`SendMsgReply`][`magic::KnownOperation::SendMsgReply`]
    pub operation: Operation,
    /// packet 对应的 数据，大部分都应该是 json string
    pub body: String,
}

impl Message {
    /// 生成一个 auth 包
    pub fn auth(room_id: u64, token: &str) -> Self {
        let payload = serde_json::json!({
            "uid": 0,
            "roomid": room_id,
            "protover": 2,
            "platform": "web",
            "clientver": "1.14.3",
            "type": 2,
            "key": token
        });
        let body = serde_json::to_string(&payload).unwrap();

        Self {
            operation: Operation::Known(magic::KnownOperation::Auth),
            body,
        }
    }
    /// 生成一个心跳包
    pub fn heartbeat() -> Message {
        Message {
            operation: Operation::Known(magic::KnownOperation::Heartbeat),
            body: "{}".to_string(),
        }
    }

    /// 从 bytes 解析出一堆 [`Message`]
    pub fn from_bytes(bytes: &[u8]) -> Result<Vec<Message>, ParseError> {
        use byteorder::{BigEndian, ReadBytesExt};
        use std::io::Read;

        let mut messages = vec![];
        // parse bytes to messages
        let mut buffer: &[u8] = bytes;
        while !buffer.is_empty() {
            let total_size = buffer.read_u32::<BigEndian>()?;
            let _raw_header_size = buffer.read_u16::<BigEndian>()?;
            let ver = buffer.read_u16::<BigEndian>()?;
            let operation = buffer.read_u32::<BigEndian>()?;
            let operation = Operation::from(operation);
            let seq_id = buffer.read_u32::<BigEndian>()?;
            trace!("seq_id = {}", seq_id);
            // read rest data
            let offset = total_size as usize - magic::HEADER_SIZE;

            let body_buffer = &buffer[..offset];

            match (operation, ver) {
                (_, magic::VER_ZLIB_COMPRESSED) => {
                    trace!(
                        "ver = VER_ZLIB_COMPRESSED, op = {:?}, trying decompress",
                        operation
                    );
                    let mut z = flate2::read::ZlibDecoder::new(body_buffer);
                    let mut buffer = vec![];
                    let bytes_read = z.read_to_end(&mut buffer)?;
                    trace!("read {} bytes from zlib", bytes_read);
                    // 居然还要递归
                    let sub_messages = Self::from_bytes(&buffer).map_err(|e| match e {
                        ParseError::Encoding(e) => {
                            debug!("utf8 decoded error, raw bytes = {:?}", bytes);
                            e.into()
                        }
                        e => e,
                    })?;
                    messages.extend(sub_messages);
                }
                (Operation::Known(magic::KnownOperation::HeartbeatReply), magic::VER_NORMAL) => {
                    // 烦不烦，能不能统一返回 string
                    let mut body_buffer = body_buffer;
                    let popularity = body_buffer.read_u32::<BigEndian>()?;
                    let message = Message {
                        operation,
                        body: popularity.to_string(),
                    };
                    messages.push(message);
                }
                (operation, ver) => {
                    let body = match String::from_utf8(body_buffer.to_vec()) {
                        Ok(body) => body,
                        Err(e) => {
                            debug!("utf8 decoded error, raw bytes = {:?}", bytes);
                            warn!(
                                "Failed to parse body as utf8, op = {:?}, ver = {:?}",
                                operation, ver
                            );
                            return Err(e.into());
                        }
                    };

                    let message = Message { operation, body };
                    messages.push(message);
                }
            }

            buffer = &buffer[offset..];
        }
        Ok(messages)
    }

    /// 从 [`WsMessage`] 解析出一堆 [`Message`]
    pub fn from_ws_message(ws_message: WsMessage) -> Result<Vec<Message>, ParseError> {
        match ws_message {
            WsMessage::Binary(bytes) => Self::from_bytes(&bytes),
            ws_message => {
                warn!("Unknown type of websocket message: {:?}", ws_message);
                Err(ParseError::WsTypeNotSupported(ws_message.to_string()))
            }
        }
    }
}

impl From<Message> for WsMessage {
    fn from(msg: Message) -> WsMessage {
        use byteorder::{BigEndian, WriteBytesExt};

        let body_size = msg.body.len();
        let total_size = magic::HEADER_SIZE + body_size;

        let mut buffer = vec![0; magic::HEADER_SIZE];
        buffer.extend_from_slice(msg.body.as_bytes());

        let mut cursor = std::io::Cursor::new(buffer);

        cursor.write_u32::<BigEndian>(total_size as u32).unwrap();
        cursor
            .write_u16::<BigEndian>(magic::HEADER_SIZE as u16)
            .unwrap();
        cursor.write_u16::<BigEndian>(1u16).unwrap();
        cursor.write_u32::<BigEndian>(msg.operation.into()).unwrap();
        cursor.write_u32::<BigEndian>(1u32).unwrap();

        let bytes = cursor.into_inner();
        WsMessage::Binary(bytes)
    }
}
