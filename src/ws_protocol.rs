use serde::Serialize;

pub mod magic {

    /// I    |      H    |  H  |  I  |  I
    /// u32  |     u16   | u16 | u32 | u32
    /// size | head size |  1  |  op |  1
    pub const HEADER_SIZE: usize = 4 + 2 + 2 + 4 + 4;
}

// # go-common\app\service\main\broadcast\model\operation.go
#[repr(u32)]
pub enum Operations {
    Handshake = 0,
    HandshakeReply = 1,
    Heartbeat = 2,
    HeartbeatReply = 3,
    SendMsg = 4,
    SendMsgReply = 5,
    DISCONNECTReply = 6,
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
impl Operations {
    pub fn make<T: Serialize>(self, body: &T) -> Vec<u8> {
        use byteorder::{BigEndian, WriteBytesExt};

        let body = serde_json::to_string(&body).expect("Failed to serialize body as json.");
        let body_size = body.len();
        let total_size = magic::HEADER_SIZE + body_size;

        let mut buffer = vec![0; magic::HEADER_SIZE];
        buffer.extend_from_slice(body.as_bytes());

        let mut cursor = std::io::Cursor::new(buffer);

        cursor.write_u32::<BigEndian>(total_size as u32).unwrap();
        cursor
            .write_u16::<BigEndian>(magic::HEADER_SIZE as u16)
            .unwrap();
        cursor.write_u16::<BigEndian>(1u16).unwrap();
        cursor.write_u32::<BigEndian>(self as u32).unwrap();
        cursor.write_u32::<BigEndian>(1u32).unwrap();

        cursor.into_inner()
    }
}
