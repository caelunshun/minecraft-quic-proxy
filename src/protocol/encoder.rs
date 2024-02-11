/// A raw encoder for a Minecraft bitstream.
#[derive(Debug)]
pub struct Encoder<'a> {
    buffer: &'a mut Vec<u8>,
}

impl<'a> Encoder<'a> {
    /// Creates an encoder that will append to the provided
    /// byte buffer.
    ///
    /// Any existing contents of `buffer` are left untouched.
    pub fn new(buffer: &'a mut Vec<u8>) -> Self {
        Self { buffer }
    }

    /// Writes an unsigned byte to the stream.
    pub fn write_u8(&mut self, x: u8) {
        self.buffer.push(x);
    }

    /// Writes a signed byte to the stream.
    pub fn write_i8(&mut self, x: i8) {
        self.write_u8(bytemuck::cast(x));
    }

    /// Writes an unsigned short to the stream.
    pub fn write_u16(&mut self, x: u16) {
        self.buffer.extend(x.to_be_bytes());
    }

    /// Writes a signed short to the stream.
    pub fn write_i16(&mut self, x: i16) {
        self.buffer.extend(x.to_be_bytes());
    }

    /// Writes an unsigned int to the stream.
    pub fn write_u32(&mut self, x: u32) {
        self.buffer.extend(x.to_be_bytes());
    }

    /// Writes a signed int to the stream.
    pub fn write_i32(&mut self, x: i32) {
        self.buffer.extend(x.to_be_bytes());
    }

    /// Writes an unsigned long to the stream.
    pub fn write_u64(&mut self, x: u64) {
        self.buffer.extend(x.to_be_bytes());
    }

    /// Writes a signed long to the stream.
    pub fn write_i64(&mut self, x: i64) {
        self.buffer.extend(x.to_be_bytes());
    }

    /// Writes a float to the stream.
    pub fn write_f32(&mut self, x: f32) {
        self.buffer.extend(x.to_be_bytes());
    }

    /// Writes a double to the stream.
    pub fn write_f64(&mut self, x: f64) {
        self.buffer.extend(x.to_be_bytes());
    }

    /// Writes a boolean to the stream.
    pub fn write_bool(&mut self, x: bool) {
        self.write_u8(if x { 0x01 } else { 0x00 });
    }

    /// Writes a series of bytes to the stream. Does not write
    /// any sort of length prefix.
    pub fn write_slice(&mut self, slice: &[u8]) {
        self.buffer.extend_from_slice(slice);
    }

    /// Writes a VarInt to the stream. Returns the number of bytes written.
    pub fn write_var_int(&mut self, x: i32) -> usize {
        let mut x: u32 = bytemuck::cast(x);
        let mut bytes_written = 0;
        loop {
            let mut temp = (x & 0b0111_1111) as u8;
            x >>= 7;
            if x != 0 {
                temp |= 0b1000_0000;
            }

            self.buffer.push(temp);
            bytes_written += 1;

            if x == 0 {
                break bytes_written;
            }
        }
    }

    /// Writes a varint-prefixed string to the stream.
    pub fn write_string(&mut self, x: &str) {
        self.write_var_int(x.len().try_into().unwrap_or(i32::MAX));
        self.buffer.extend_from_slice(x.as_bytes());
    }

    /// Writes a fixed-point-encoded angle to the stream.
    pub fn write_angle(&mut self, degrees: f32) {
        let x = (degrees / 360.0 * u8::MAX as f32).round() as u8;
        self.buffer.push(x);
    }
}

/// A type that can be written to an [`Encoder`].
pub trait Encode {
    fn encode(&self, encoder: &mut Encoder);
}

impl Encode for u8 {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_u8(*self);
    }
}

impl Encode for i8 {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_i8(*self);
    }
}

impl Encode for u16 {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_u16(*self);
    }
}

impl Encode for i16 {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_i16(*self);
    }
}

impl Encode for u32 {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_u32(*self);
    }
}

impl Encode for i32 {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_i32(*self);
    }
}

impl Encode for u64 {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_u64(*self);
    }
}

impl Encode for i64 {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_i64(*self);
    }
}

impl Encode for f32 {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_f32(*self);
    }
}

impl Encode for f64 {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_f64(*self);
    }
}

impl Encode for bool {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_bool(*self);
    }
}

impl Encode for String {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_string(self);
    }
}

impl Encode for u128 {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_slice(&self.to_be_bytes())
    }
}

impl Encode for () {
    fn encode(&self, _encoder: &mut Encoder) {}
}
