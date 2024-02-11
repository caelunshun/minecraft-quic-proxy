use crate::position::BlockPosition;
use std::{backtrace::Backtrace, convert::Infallible, num::TryFromIntError, str::Utf8Error};

/// An error while decoding packets.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("need at least {0} more bytes at {1}")]
    EndOfStream(usize, Backtrace),
    #[error("invalid boolean pattern {0} - expected either 0 or 1")]
    InvalidBool(u8),
    #[error("varint / varlong is too long")]
    VarIntTooLong,
    #[error("string exceeds max allowed length")]
    StringTooLong,
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
    #[error(transparent)]
    IntConversion(#[from] TryFromIntError),
    /// Special variant for derive macro integer conversions to work.
    /// Cannot occur.
    #[error(transparent)]
    Infallible(#[from] Infallible),
    #[error(transparent)]
    Other(
        #[from]
        #[backtrace]
        anyhow::Error,
    ),
}

pub type Result<T, E = DecodeError> = std::result::Result<T, E>;

const MAX_STRING_LENGTH: usize = i16::MAX as usize;

/// A raw decoder for a Minecraft bitstream.
#[derive(Debug)]
pub struct Decoder<'a> {
    buffer: &'a [u8],
}

impl<'a> Decoder<'a> {
    /// Creates a decoder from the buffer it will read from.
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer }
    }

    /// Creates a new decoder at the same position.
    pub fn duplicate(&self) -> Self {
        Self {
            buffer: self.buffer,
        }
    }

    /// Gets the remaining buffer.
    pub fn buffer(&self) -> &'a [u8] {
        self.buffer
    }

    /// Returns if there is no data left in the buffer.
    pub fn is_finished(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Consumes `n` bytes from the buffer, returning them as a slice.
    pub fn consume_slice(&mut self, n: usize) -> Result<&'a [u8]> {
        if n <= self.buffer.len() {
            let (data, buffer) = self.buffer.split_at(n);
            self.buffer = buffer;
            Ok(data)
        } else {
            Err(DecodeError::EndOfStream(n, Backtrace::capture()))
        }
    }

    /// Consumes `N` bytes into an array.
    pub fn consume<const N: usize>(&mut self) -> Result<[u8; N]> {
        let data = self.consume_slice(N)?;
        Ok(<[u8; N]>::try_from(data).unwrap())
    }

    /// Reads an unsigned byte from the stream.
    pub fn read_u8(&mut self) -> Result<u8> {
        self.consume::<1>().map(|[x]| x)
    }

    /// Reads a signed byte from the stream.
    pub fn read_i8(&mut self) -> Result<i8> {
        self.consume().map(i8::from_be_bytes)
    }

    /// Reads an unsigned short from the stream.
    pub fn read_u16(&mut self) -> Result<u16> {
        self.consume().map(u16::from_be_bytes)
    }

    /// Reads a signed short from the stream.
    pub fn read_i16(&mut self) -> Result<i16> {
        self.consume().map(i16::from_be_bytes)
    }

    /// Reads an unsigned int from the stream.
    pub fn read_u32(&mut self) -> Result<u32> {
        self.consume().map(u32::from_be_bytes)
    }

    /// Reads a signed int from the stream.
    pub fn read_i32(&mut self) -> Result<i32> {
        self.consume().map(i32::from_be_bytes)
    }

    /// Reads an unsigned long from the stream.
    pub fn read_u64(&mut self) -> Result<u64> {
        self.consume().map(u64::from_be_bytes)
    }

    /// Reads a signed long from the stream.
    pub fn read_i64(&mut self) -> Result<i64> {
        self.consume().map(i64::from_be_bytes)
    }

    /// Reads a float from the stream.
    pub fn read_f32(&mut self) -> Result<f32> {
        self.consume().map(f32::from_be_bytes)
    }

    /// Reads a double from the stream.
    pub fn read_f64(&mut self) -> Result<f64> {
        self.consume().map(f64::from_be_bytes)
    }

    /// Reads a boolean from the stream.
    pub fn read_bool(&mut self) -> Result<bool> {
        let x = self.read_u8()?;
        match x {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::InvalidBool(x)),
        }
    }

    /// Reads a VarInt from the stream.
    pub fn read_var_int(&mut self) -> Result<i32> {
        self.read_var_int_with_size().map(|(x, _)| x)
    }

    /// Reads a VarInt from the stream, additionally
    /// returning the number of bytes read.
    pub fn read_var_int_with_size(&mut self) -> Result<(i32, usize)> {
        let mut num_read = 0;
        let mut result = 0;

        loop {
            let read = self.read_u8()?;
            let value = i32::from(read & 0b0111_1111);
            result |= value.overflowing_shl(7 * num_read).0;

            num_read += 1;

            if num_read > 5 {
                return Err(DecodeError::VarIntTooLong);
            }
            if read & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok((result, num_read as usize))
    }

    pub fn read_block_position(&mut self) -> Result<BlockPosition> {
        let value = self.read_i64()?;

        let x = (value >> 38) as i32;
        let y = (value & 0xFFF) as i32;
        let z = (value << 26 >> 38) as i32;

        Ok(BlockPosition { x, y, z })
    }

    /// Reads a string from the stream.
    pub fn read_string(&mut self) -> Result<&'a str> {
        let length = usize::try_from(self.read_var_int()?)?;

        if length > MAX_STRING_LENGTH {
            return Err(DecodeError::StringTooLong);
        }

        let bytes = std::str::from_utf8(self.consume_slice(length)?)?;
        Ok(bytes)
    }

    pub fn read_angle(&mut self) -> Result<f32> {
        let fixed = self.read_u8()?;
        Ok((fixed as f32 / u8::MAX as f32) * 360.)
    }
}

/// A type that can be read from a [`Decoder`].
pub trait Decode: Sized {
    fn decode(decoder: &mut Decoder) -> Result<Self>;
}

impl Decode for u8 {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        decoder.read_u8()
    }
}

impl Decode for i8 {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        decoder.read_i8()
    }
}

impl Decode for u16 {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        decoder.read_u16()
    }
}

impl Decode for i16 {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        decoder.read_i16()
    }
}

impl Decode for u32 {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        decoder.read_u32()
    }
}

impl Decode for i32 {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        decoder.read_i32()
    }
}

impl Decode for u64 {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        decoder.read_u64()
    }
}

impl Decode for i64 {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        decoder.read_i64()
    }
}

impl Decode for f32 {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        decoder.read_f32()
    }
}

impl Decode for f64 {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        decoder.read_f64()
    }
}

impl Decode for bool {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        decoder.read_bool()
    }
}

impl Decode for String {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        decoder.read_string().map(str::to_owned)
    }
}

impl Decode for u128 {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        let bytes = decoder.consume::<16>()?;
        Ok(Self::from_be_bytes(bytes))
    }
}

impl Decode for BlockPosition {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        decoder.read_block_position()
    }
}

impl Decode for () {
    fn decode(_decoder: &mut Decoder) -> Result<Self> {
        Ok(())
    }
}
