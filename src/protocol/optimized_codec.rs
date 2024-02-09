//! Alternative codec implementation designed for use over QUIC.
//!
//! The format is as follows:
//! 1. VarInt - size of rest of packet, in bytes
//! 2. 1 byte flags: 0x01 = compressed
//! 3. Packet bytes. Compressed with `zstd` if the compression flag is set.
//!
//! Compared to the vanilla codec, there is
//! * no encryption - QUIC handles this for us
//! * no compression enabled/disabled state - compression is always used for large packets
//! * a codec instance for each stream rather than a single shared one
//!
//! Future improvements:
//! * use a pre-trained dictionary for better compression

use crate::protocol::{
    packet, packet::ProtocolState, vanilla_codec::var_int_size, Decode, Decoder, Encode, Encoder,
    BUFFER_LIMIT,
};
use anyhow::{bail, Context};
use bitflags::bitflags;
use std::{marker::PhantomData, mem::size_of};
use zstd::{
    bulk::{Compressor, Decompressor},
    zstd_safe::CompressionLevel,
};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct Flags: u8 {
        const COMPRESSED = 0x01;
    }
}

/// Use a high compression value to reduce bandwidth usage over the QUIC connection.
const COMPRESSION_LEVEL: CompressionLevel = 12;

/// Codec implementation for packets sent over QUIC.
///
/// Interface is the same as for `VanillaCodec`.
pub struct OptimizedCodec<Side, State> {
    read_buffer: Vec<u8>,
    compressor: Compressor<'static>,
    decompressor: Decompressor<'static>,
    _marker: PhantomData<(Side, State)>,
}

impl<Side, State> OptimizedCodec<Side, State>
where
    Side: packet::Side,
    State: ProtocolState,
{
    pub fn new() -> Self {
        let mut compressor = Compressor::new(COMPRESSION_LEVEL).expect("failed to initialize zstd");
        let mut decompressor = Decompressor::new().expect("failed to initialize zstd");
        compressor.include_checksum(false).unwrap();
        compressor.include_contentsize(false).unwrap();
        compressor.include_dictid(false).unwrap();
        compressor.include_magicbytes(false).unwrap();

        decompressor.include_magicbytes(false).unwrap();

        Self {
            read_buffer: Vec::new(),
            compressor,
            decompressor,
            _marker: PhantomData,
        }
    }

    pub fn switch_state<NewState: ProtocolState>(self) -> OptimizedCodec<Side, NewState> {
        OptimizedCodec {
            read_buffer: self.read_buffer,
            compressor: self.compressor,
            decompressor: self.decompressor,
            _marker: PhantomData,
        }
    }

    pub fn encode_packet(&mut self, packet: &Side::SendPacket<State>) -> anyhow::Result<Vec<u8>> {
        let mut plain_data = Vec::new();
        packet.encode(&mut Encoder::new(&mut plain_data));

        const COMPRESSION_THRESHOLD: usize = 128;
        let should_compress = plain_data.len() >= COMPRESSION_THRESHOLD;
        let mut flags = Flags::empty();
        let encoded_data = if should_compress {
            flags |= Flags::COMPRESSED;
            self.compressor.compress(&plain_data)?
        } else {
            plain_data
        };

        let mut result_buf = Vec::new();
        let mut encoder = Encoder::new(&mut result_buf);

        let flag_len = size_of::<u8>();
        let len = encoded_data.len() + flag_len;
        encoder.write_var_int(len.try_into()?);

        encoder.write_u8(flags.bits());
        encoder.write_slice(&encoded_data);

        Ok(result_buf)
    }

    pub fn give_data(&mut self, data: &[u8]) {
        self.read_buffer.extend_from_slice(data);
    }

    pub fn decode_packet(&mut self) -> anyhow::Result<Option<Side::RecvPacket<State>>> {
        let mut decoder = Decoder::new(&self.read_buffer);
        let length = usize::try_from(decoder.read_var_int()?)?;
        if length > BUFFER_LIMIT {
            bail!("packet length of {length} is too large");
        }

        let total_bytes_read = var_int_size(length as i32) + length;

        let remaining_data = decoder.buffer();
        if remaining_data.len() < length {
            return Ok(None);
        }
        let data = &remaining_data[..length];

        let mut decoder = Decoder::new(data);
        let flags = Flags::from_bits(decoder.read_u8()?).context("invalid flags")?;
        let result = if flags.contains(Flags::COMPRESSED) {
            let decompressed = self
                .decompressor
                .decompress(decoder.buffer(), BUFFER_LIMIT)?;
            let packet = Side::RecvPacket::<State>::decode(&mut Decoder::new(&decompressed))?;
            Ok(Some(packet))
        } else {
            let packet = Side::RecvPacket::<State>::decode(&mut decoder)?;
            Ok(Some(packet))
        };

        self.read_buffer.drain(..total_bytes_read);
        result
    }
}
