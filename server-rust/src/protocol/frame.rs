use std::fmt::Debug;

pub const MAGIC_HEADER: u16 = 0x5359; // "SY"
pub const PROTOCOL_VERSION: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    Handshake = 0x01,
    Publish = 0x02,
    Message = 0x03,
    Ack = 0x04,
    SyncRequest = 0x05,
    SyncResponse = 0x06,
    HeartbeatPing = 0x07,
    HeartbeatPong = 0x08,
}

impl OpCode {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x01 => Some(OpCode::Handshake),
            0x02 => Some(OpCode::Publish),
            0x03 => Some(OpCode::Message),
            0x04 => Some(OpCode::Ack),
            0x05 => Some(OpCode::SyncRequest),
            0x06 => Some(OpCode::SyncResponse),
            0x07 => Some(OpCode::HeartbeatPing),
            0x08 => Some(OpCode::HeartbeatPong),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynapseFrame {
    pub opcode: OpCode,
    pub sequence: u32,
    pub channel_id: String,
    pub payload: Vec<u8>,
}

impl SynapseFrame {
    pub fn new(opcode: OpCode, sequence: u32, channel_id: &str, payload: Vec<u8>) -> Self {
        Self {
            opcode,
            sequence,
            channel_id: channel_id.to_string(),
            payload,
        }
    }
}
