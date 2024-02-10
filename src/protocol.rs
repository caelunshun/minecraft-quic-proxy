//! Implements the Minecraft protocol.

pub const PROTOCOL_VERSION: i32 = 765; // 1.20.4

pub mod decoder;
pub mod encoder;
pub mod optimized_codec;
pub mod packet;
pub mod vanilla_codec;

pub use decoder::{Decode, DecodeError, Decoder};
pub use encoder::{Encode, Encoder};

/// Limit to avoid out-of-memory DOS.
const BUFFER_LIMIT: usize = 1024 * 1024; // 1 MiB
