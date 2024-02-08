pub const PROTOCOL_VERSION: i32 = 765; // 1.20.4

mod decoder;
mod encoder;
mod packet;
mod optimized_codec;
mod vanilla_codec;

pub use decoder::{Decode, DecodeError, Decoder};
pub use encoder::{Encode, Encoder};

/// Limit to avoid out-of-memory DOS.
const BUFFER_LIMIT: usize = 1024 * 1024; // 1 MiB

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ConnectionState {
    Handshake,
    Status,
    Configuration,
    Play,
}
