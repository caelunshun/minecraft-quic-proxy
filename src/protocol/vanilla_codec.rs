//! Codec implementation for the vanilla codec.
//! Supports zlib compression and CFB8 encryption.

use super::BUFFER_LIMIT;
use crate::protocol::{
    packet, packet::ProtocolState, Decode, DecodeError, Decoder, Encode, Encoder,
};
use aes::{cipher::generic_array::GenericArray, Aes128};
use anyhow::bail;
use cfb8::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use flate2::Compression;
use std::{
    borrow::Cow,
    io::{Read, Write},
    marker::PhantomData,
    num::NonZeroUsize,
    slice,
};

/// Since the proxy will rarely sent large amounts of compressed data
/// over the vanilla codec, we avoid spending too much time on compression here.
/// (This is because most or even all serverbound packets are very small.)
const COMPRESSION_LEVEL: Compression = Compression::fast();

/// Key used for encryption.
#[derive(Copy, Clone, Debug)]
pub struct EncryptionKey([u8; 16]);

impl EncryptionKey {
    pub fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

/// Threshold in bytes where a packet will be compressed.
#[derive(Copy, Clone, Debug)]
pub struct CompressionThreshold(NonZeroUsize);

impl CompressionThreshold {
    pub fn new(threshold: NonZeroUsize) -> Self {
        Self(threshold)
    }
}

/// Codec state.
pub struct VanillaCodec<Side, State> {
    /// Buffered incoming bytes.
    read_buffer: Vec<u8>,
    encryption_state: Option<EncryptionState>,
    compression_state: Option<CompressionState>,
    _marker: PhantomData<(Side, State)>,
}

impl<Side, State> VanillaCodec<Side, State>
where
    Side: packet::Side,
    State: ProtocolState,
{
    pub fn new() -> Self {
        Self {
            read_buffer: Vec::new(),
            encryption_state: None,
            compression_state: None,
            _marker: PhantomData,
        }
    }

    pub fn switch_state<NewState: ProtocolState>(self) -> VanillaCodec<Side, NewState> {
        VanillaCodec {
            read_buffer: self.read_buffer,
            encryption_state: self.encryption_state,
            compression_state: self.compression_state,
            _marker: PhantomData,
        }
    }

    pub fn enable_encryption(&mut self, key: EncryptionKey) {
        assert!(
            self.encryption_state.is_none(),
            "called enable_encryption() multiple times"
        );
        self.encryption_state = Some(EncryptionState {
            encryptor: cfb8::Encryptor::new(&key.0.into(), &key.0.into()),
            decryptor: cfb8::Decryptor::new(&key.0.into(), &key.0.into()),
        });
    }

    pub fn enable_compression(&mut self, threshold: CompressionThreshold) {
        assert!(
            self.compression_state.is_none(),
            "called enable_compression() multiple times"
        );
        self.compression_state = Some(CompressionState { threshold });
    }

    /// Encodes a packet to a stream of bytes in the protocol format.
    pub fn encode_packet(&mut self, packet: &Side::SendPacket<State>) -> anyhow::Result<Vec<u8>> {
        let mut plain_buf = Vec::new();
        packet.encode(&mut Encoder::new(&mut plain_buf));

        let uncompressed_length = i32::try_from(plain_buf.len())?;
        let mut compressed_buf = match &self.compression_state {
            Some(CompressionState { threshold }) => {
                let (data_length, compressed_data) =
                    if uncompressed_length as usize >= threshold.0.get() {
                        let mut encoder =
                            flate2::write::ZlibEncoder::new(Vec::new(), COMPRESSION_LEVEL);
                        encoder.write_all(&plain_buf).expect("infallible write");
                        (uncompressed_length, encoder.finish()?)
                    } else {
                        // send uncompressed
                        (0, plain_buf)
                    };
                let mut buf = Vec::new();
                let mut encoder = Encoder::new(&mut buf);
                encoder.write_var_int(
                    var_int_size(data_length) as i32 + i32::try_from(compressed_data.len())?,
                );

                buf
            }
            None => {
                let mut buf = Vec::new();
                let mut encoder = Encoder::new(&mut buf);
                encoder.write_var_int(uncompressed_length);
                encoder.write_slice(&plain_buf);
                buf
            }
        };

        match &mut self.encryption_state {
            Some(EncryptionState { encryptor, .. }) => {
                for x in &mut compressed_buf {
                    let slice = slice::from_mut(x);
                    encryptor.encrypt_block_mut(GenericArray::from_mut_slice(slice));
                }
            }
            None => {}
        }

        Ok(compressed_buf)
    }

    /// Gives data to the internal read buffer.
    ///
    /// `data` will be modified in-place and its results
    /// after calling this function are unspecified.
    ///
    /// Call `decode_packet` to get a packet.
    pub fn give_data(&mut self, mut data: impl AsMut<[u8]>) {
        let data = data.as_mut();
        if let Some(EncryptionState { decryptor, .. }) = &mut self.encryption_state {
            for byte in data.iter_mut() {
                let slice = slice::from_mut(byte);
                decryptor.decrypt_block_mut(GenericArray::from_mut_slice(slice));
            }
        }

        self.read_buffer.extend_from_slice(data);
    }

    /// Attempts to decode a packet.
    /// This should be called in a loop after any call to `give_data`
    /// until this function returns `None`.
    ///
    /// * If not enough data is available, returns `Ok(None)`.
    /// * If a packet was read, returns `Ok(Some(packet))`. More packets may be available.
    /// * If an error occurs, returns `Err(e)`, invalidating the stream.
    pub fn decode_packet(&mut self) -> anyhow::Result<Option<Side::RecvPacket<State>>> {
        // Note: data in the read buffer is already decrypted.
        let mut decoder = Decoder::new(&self.read_buffer);
        let length = decoder.read_var_int()?;
        let length = usize::try_from(length)?;
        let total_bytes = length + var_int_size(length as i32);

        if length > BUFFER_LIMIT {
            bail!("packet length of {length} exceeds maximum allowed");
        }
        let packet_contents = match decoder.consume_slice(length) {
            Ok(x) => x,
            Err(DecodeError::EndOfStream(_, _)) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        let plain_data = match &self.compression_state {
            Some(_) => {
                let mut decoder = Decoder::new(packet_contents);
                let uncompressed_length = usize::try_from(decoder.read_var_int()?)?;
                if uncompressed_length == 0 {
                    Cow::Borrowed(decoder.buffer())
                } else {
                    let mut buf = Vec::new();
                    flate2::read::ZlibDecoder::new(decoder.buffer())
                        .take(BUFFER_LIMIT.try_into().unwrap())
                        .read_to_end(&mut buf)?;
                    Cow::Owned(buf)
                }
            }
            None => Cow::Borrowed(packet_contents),
        };

        let packet = Side::RecvPacket::<State>::decode(&mut Decoder::new(&plain_data))?;
        self.read_buffer.drain(..total_bytes);
        Ok(Some(packet))
    }
}

struct EncryptionState {
    encryptor: cfb8::Encryptor<Aes128>,
    decryptor: cfb8::Decryptor<Aes128>,
}

struct CompressionState {
    threshold: CompressionThreshold,
}

pub fn var_int_size(x: i32) -> usize {
    Encoder::new(&mut Vec::new()).write_var_int(x)
}
