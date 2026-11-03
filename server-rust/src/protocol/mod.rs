pub mod frame;
pub mod codec;

pub use frame::{SynapseFrame, OpCode, MAGIC_HEADER, PROTOCOL_VERSION};
pub use codec::{SynapseCodec, CodecError, crc32_ieee};
