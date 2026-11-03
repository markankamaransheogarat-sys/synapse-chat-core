use super::frame::{OpCode, SynapseFrame, MAGIC_HEADER, PROTOCOL_VERSION};

#[derive(Debug, PartialEq, Eq)]
pub enum CodecError {
    IncompleteFrame,
    InvalidMagic,
    UnsupportedVersion(u8),
    UnknownOpCode(u8),
    ChecksumMismatch { expected: u32, computed: u32 },
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodecError::IncompleteFrame => write!(f, "Incomplete frame buffer"),
            CodecError::InvalidMagic => write!(f, "Invalid magic sync header"),
            CodecError::UnsupportedVersion(v) => write!(f, "Unsupported protocol version: {}", v),
            CodecError::UnknownOpCode(op) => write!(f, "Unknown opcode: {:#x}", op),
            CodecError::ChecksumMismatch { expected, computed } => {
                write!(f, "CRC32 checksum mismatch (expected: {:#x}, computed: {:#x})", expected, computed)
            }
        }
    }
}

impl std::error::Error for CodecError {}

/// Standard IEEE 802.3 CRC-32 implementation (zero-dependency)
pub fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1) != 0;
            crc >>= 1;
            if mask {
                crc ^= 0xEDB8_8320;
            }
        }
    }
    !crc
}

pub struct SynapseCodec;

impl SynapseCodec {
    /// Encode a SynapseFrame into binary buffer
    pub fn encode(frame: &SynapseFrame) -> Vec<u8> {
        let channel_bytes = frame.channel_id.as_bytes();
        let channel_len = channel_bytes.len().min(255) as u8;
        let payload_len = frame.payload.len() as u32;

        let mut buf = Vec::with_capacity(12 + channel_len as usize + frame.payload.len() + 4);

        // Header: Magic (2) + Version (1) + OpCode (1) + Sequence (4) + ChannelLen (1)
        buf.extend_from_slice(&MAGIC_HEADER.to_be_bytes());
        buf.push(PROTOCOL_VERSION);
        buf.push(frame.opcode as u8);
        buf.extend_from_slice(&frame.sequence.to_be_bytes());
        buf.push(channel_len);
        buf.extend_from_slice(&channel_bytes[..channel_len as usize]);

        // Payload Len (4) + Payload
        buf.extend_from_slice(&payload_len.to_be_bytes());
        buf.extend_from_slice(&frame.payload);

        // Compute CRC-32 over everything before checksum
        let checksum = crc32_ieee(&buf);
        buf.extend_from_slice(&checksum.to_be_bytes());

        buf
    }

    /// Decode a binary buffer into SynapseFrame
    pub fn decode(src: &[u8]) -> Result<(SynapseFrame, usize), CodecError> {
        // Minimum frame size: Magic(2) + Ver(1) + Op(1) + Seq(4) + ChanLen(1) + PayLen(4) + CRC(4) = 17 bytes
        if src.len() < 17 {
            return Err(CodecError::IncompleteFrame);
        }

        let magic = u16::from_be_bytes([src[0], src[1]]);
        if magic != MAGIC_HEADER {
            return Err(CodecError::InvalidMagic);
        }

        let version = src[2];
        if version != PROTOCOL_VERSION {
            return Err(CodecError::UnsupportedVersion(version));
        }

        let opcode = OpCode::from_u8(src[3]).ok_or(CodecError::UnknownOpCode(src[3]))?;
        let sequence = u32::from_be_bytes([src[4], src[5], src[6], src[7]]);
        let channel_len = src[8] as usize;

        let channel_end = 9 + channel_len;
        if src.len() < channel_end + 4 + 4 {
            return Err(CodecError::IncompleteFrame);
        }

        let channel_id = String::from_utf8_lossy(&src[9..channel_end]).to_string();

        let payload_len = u32::from_be_bytes([
            src[channel_end],
            src[channel_end + 1],
            src[channel_end + 2],
            src[channel_end + 3],
        ]) as usize;

        let payload_end = channel_end + 4 + payload_len;
        let total_frame_len = payload_end + 4;

        if src.len() < total_frame_len {
            return Err(CodecError::IncompleteFrame);
        }

        let payload = src[channel_end + 4..payload_end].to_vec();

        let expected_checksum = u32::from_be_bytes([
            src[payload_end],
            src[payload_end + 1],
            src[payload_end + 2],
            src[payload_end + 3],
        ]);

        let computed_checksum = crc32_ieee(&src[..payload_end]);
        if computed_checksum != expected_checksum {
            return Err(CodecError::ChecksumMismatch {
                expected: expected_checksum,
                computed: computed_checksum,
            });
        }

        Ok((
            SynapseFrame {
                opcode,
                sequence,
                channel_id,
                payload,
            },
            total_frame_len,
        ))
    }
}
