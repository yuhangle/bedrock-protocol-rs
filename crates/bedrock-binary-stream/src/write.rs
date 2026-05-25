use bedrock_common::{BinaryStreamError, BedrockWrite};

/// A writable binary stream that owns its internal `Vec<u8>` buffer.
///
/// Supports writing all types used by the Minecraft Bedrock Edition protocol.
/// After writing, the buffer can be accessed via `data()` or consumed via `into_data()`.
pub struct BinaryStream {
    buffer: Vec<u8>,
    big_endian: bool,
}

impl BinaryStream {
    /// Create a new empty stream.
    pub fn new(big_endian: bool) -> Self {
        Self {
            buffer: Vec::new(),
            big_endian,
        }
    }

    /// Create a stream from an existing buffer. Writes append to the end.
    pub fn from_vec(buffer: Vec<u8>, big_endian: bool) -> Self {
        Self { buffer, big_endian }
    }

    /// Reserve capacity for at least `capacity` bytes.
    pub fn reserve(&mut self, capacity: usize) {
        self.buffer.reserve(capacity);
    }

    /// Reset the stream to its initial empty state.
    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    /// Return a reference to the internal buffer.
    pub fn data(&self) -> &[u8] {
        &self.buffer
    }

    /// Return a copy of the internal buffer.
    pub fn copy_buffer(&self) -> Vec<u8> {
        self.buffer.clone()
    }

    /// Consume the stream and return the internal buffer.
    pub fn into_data(self) -> Vec<u8> {
        self.buffer
    }

    /// Take the internal buffer, replacing it with empty.
    pub fn get_and_release_data(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.buffer)
    }

    /// Write the contents of a `ReadOnlyBinaryStream` into this stream.
    pub fn write_stream(&mut self, other: &crate::ReadOnlyBinaryStream) -> Result<(), BinaryStreamError> {
        self.buffer.extend_from_slice(other.as_slice());
        Ok(())
    }

    /// Get an immutable reference to the buffer.
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    /// Return the current write position (= buffer length).
    pub fn position(&self) -> usize {
        self.buffer.len()
    }
}

impl BedrockWrite for BinaryStream {
    fn write_bool(&mut self, value: bool) -> Result<(), BinaryStreamError> {
        self.buffer.push(if value { 1 } else { 0 });
        Ok(())
    }

    fn write_u8(&mut self, value: u8) -> Result<(), BinaryStreamError> {
        self.buffer.push(value);
        Ok(())
    }

    fn write_i16(&mut self, value: i16) -> Result<(), BinaryStreamError> {
        if self.big_endian {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        } else {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }

    fn write_u16(&mut self, value: u16) -> Result<(), BinaryStreamError> {
        if self.big_endian {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        } else {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }

    fn write_i32(&mut self, value: i32) -> Result<(), BinaryStreamError> {
        if self.big_endian {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        } else {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }

    fn write_u32(&mut self, value: u32) -> Result<(), BinaryStreamError> {
        if self.big_endian {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        } else {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }

    fn write_i64(&mut self, value: i64) -> Result<(), BinaryStreamError> {
        if self.big_endian {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        } else {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }

    fn write_u64(&mut self, value: u64) -> Result<(), BinaryStreamError> {
        if self.big_endian {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        } else {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }

    fn write_f32(&mut self, value: f32) -> Result<(), BinaryStreamError> {
        if self.big_endian {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        } else {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }

    fn write_f64(&mut self, value: f64) -> Result<(), BinaryStreamError> {
        if self.big_endian {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        } else {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }

    fn write_i32_be(&mut self, value: i32) -> Result<(), BinaryStreamError> {
        self.buffer.extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    fn write_u32_be(&mut self, value: u32) -> Result<(), BinaryStreamError> {
        self.buffer.extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    fn write_u24(&mut self, value: u32) -> Result<(), BinaryStreamError> {
        if value > 0x00FF_FFFF {
            return Err(BinaryStreamError::UnsupportedValue {
                description: format!("u24 value out of range: {}", value),
            });
        }
        if self.big_endian {
            let bytes = value.to_be_bytes();
            self.buffer.extend_from_slice(&bytes[1..]);
        } else {
            let bytes = value.to_le_bytes();
            self.buffer.extend_from_slice(&bytes[..3]);
        }
        Ok(())
    }

    fn write_varint(&mut self, value: i32) -> Result<(), BinaryStreamError> {
        let encoded = bedrock_common::varint::encode_varint32(value);
        self.buffer.extend_from_slice(&encoded);
        Ok(())
    }

    fn write_varint64(&mut self, value: i64) -> Result<(), BinaryStreamError> {
        let encoded = bedrock_common::varint::encode_varint64(value);
        self.buffer.extend_from_slice(&encoded);
        Ok(())
    }

    fn write_unsigned_varint(&mut self, value: u32) -> Result<(), BinaryStreamError> {
        let encoded = bedrock_common::varint::encode_unsigned_varint(value);
        self.buffer.extend_from_slice(&encoded);
        Ok(())
    }

    fn write_unsigned_varint64(&mut self, value: u64) -> Result<(), BinaryStreamError> {
        let encoded = bedrock_common::varint::encode_unsigned_varint64(value);
        self.buffer.extend_from_slice(&encoded);
        Ok(())
    }

    fn write_normalized_f32(&mut self, value: f32) -> Result<(), BinaryStreamError> {
        let encoded = (value * 2147483648.0f32) as i64;
        self.write_varint64(encoded)
    }

    fn write_string(&mut self, value: &str) -> Result<(), BinaryStreamError> {
        let bytes = value.as_bytes();
        self.write_unsigned_varint(bytes.len() as u32)?;
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }

    fn write_short_string(&mut self, value: &str) -> Result<(), BinaryStreamError> {
        let bytes = value.as_bytes();
        if bytes.len() > u16::MAX as usize {
            return Err(BinaryStreamError::UnsupportedValue {
                description: format!("short string too long: {} bytes", bytes.len()),
            });
        }
        self.write_u16(bytes.len() as u16)?;
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }

    fn write_long_string(&mut self, value: &str) -> Result<(), BinaryStreamError> {
        let bytes = value.as_bytes();
        if bytes.len() > u32::MAX as usize {
            return Err(BinaryStreamError::UnsupportedValue {
                description: format!("long string too long: {} bytes", bytes.len()),
            });
        }
        self.write_u32(bytes.len() as u32)?;
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }

    fn write_raw_bytes(&mut self, value: &[u8]) -> Result<(), BinaryStreamError> {
        self.buffer.extend_from_slice(value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ReadOnlyBinaryStream;
    use bedrock_common::BedrockRead;

    #[test]
    fn test_write_bool() {
        let mut s = BinaryStream::new(false);
        s.write_bool(true).unwrap();
        s.write_bool(false).unwrap();
        assert_eq!(s.into_data(), vec![1, 0]);
    }

    #[test]
    fn test_write_u8() {
        let mut s = BinaryStream::new(false);
        s.write_u8(0xAB).unwrap();
        assert_eq!(s.into_data(), vec![0xAB]);
    }

    #[test]
    fn test_write_i16_le() {
        let mut s = BinaryStream::new(false);
        s.write_i16(0x1234).unwrap();
        assert_eq!(s.into_data(), vec![0x34, 0x12]);
    }

    #[test]
    fn test_write_i16_be() {
        let mut s = BinaryStream::new(true);
        s.write_i16(0x1234).unwrap();
        assert_eq!(s.into_data(), vec![0x12, 0x34]);
    }

    #[test]
    fn test_write_u32_le() {
        let mut s = BinaryStream::new(false);
        s.write_u32(0x12345678).unwrap();
        assert_eq!(s.into_data(), vec![0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_write_f32_le() {
        let mut s = BinaryStream::new(false);
        s.write_f32(7.0).unwrap();
        assert_eq!(s.into_data(), vec![0x00, 0x00, 0xE0, 0x40]);
    }

    #[test]
    fn test_write_f64_le() {
        let mut s = BinaryStream::new(false);
        s.write_f64(6.0).unwrap();
        assert_eq!(s.into_data(), vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x40]);
    }

    #[test]
    fn test_write_varint() {
        let mut s = BinaryStream::new(false);
        s.write_varint(13).unwrap();
        assert_eq!(s.into_data(), vec![0x1A]); // ZigZag(13) = 26
    }

    #[test]
    fn test_write_string() {
        let mut s = BinaryStream::new(false);
        s.write_string("17").unwrap();
        // uvarint(2) + "17"
        assert_eq!(s.into_data(), vec![0x02, 0x31, 0x37]);
    }

    #[test]
    fn test_write_i32_be() {
        let mut s = BinaryStream::new(false);
        s.write_i32_be(16).unwrap();
        assert_eq!(s.into_data(), vec![0x00, 0x00, 0x00, 0x10]);
    }

    #[test]
    fn test_write_u24_le() {
        let mut s = BinaryStream::new(false);
        s.write_u24(18).unwrap();
        assert_eq!(s.into_data(), vec![0x12, 0x00, 0x00]);
    }

    #[test]
    fn test_get_and_release_data() {
        let mut s = BinaryStream::new(false);
        s.write_u8(1).unwrap();
        s.write_u8(2).unwrap();
        let data = s.get_and_release_data();
        assert_eq!(data, vec![1, 2]);
        assert!(s.into_data().is_empty()); // was replaced
    }

    #[test]
    fn test_reserve_and_reset() {
        let mut s = BinaryStream::new(false);
        s.reserve(1024);
        s.write_u8(42).unwrap();
        assert_eq!(s.data().len(), 1);
        s.reset();
        assert!(s.into_data().is_empty());
    }

    // -----------------------------------------------------------------------
    // Round-trip tests (write then read, must produce identical values)
    // -----------------------------------------------------------------------

    #[test]
    fn test_roundtrip_u8() {
        for val in [0, 1, 0xFF, 0x80, 0x7F] {
            let mut w = BinaryStream::new(false);
            w.write_u8(val).unwrap();
            let mut r = ReadOnlyBinaryStream::from_vec(w.into_data(), false);
            assert_eq!(r.read_u8().unwrap(), val);
        }
    }

    #[test]
    fn test_roundtrip_i32() {
        for val in [0, -1, 1, i32::MAX, i32::MIN] {
            let mut w = BinaryStream::new(false);
            w.write_i32(val).unwrap();
            let mut r = ReadOnlyBinaryStream::from_vec(w.into_data(), false);
            assert_eq!(r.read_i32().unwrap(), val);
        }
    }

    #[test]
    fn test_roundtrip_varint() {
        for val in [0i32, -1, 1, i32::MAX, i32::MIN, 127, -128, 16384] {
            let mut w = BinaryStream::new(false);
            w.write_varint(val).unwrap();
            let mut r = ReadOnlyBinaryStream::from_vec(w.into_data(), false);
            assert_eq!(r.read_varint().unwrap(), val);
        }
    }

    #[test]
    fn test_roundtrip_string() {
        let long = "a".repeat(100);
        let vals = ["", "hello", "17", "你好", long.as_str()];
        for val in vals {
            let mut w = BinaryStream::new(false);
            w.write_string(val).unwrap();
            let mut r = ReadOnlyBinaryStream::from_vec(w.into_data(), false);
            assert_eq!(r.read_string().unwrap(), val);
        }
    }

    #[test]
    fn test_roundtrip_normalized_float() {
        let vals = [0.0f32, 0.5, -0.5, 1.0, -1.0, 0.25];
        for val in vals {
            let mut w = BinaryStream::new(false);
            w.write_normalized_f32(val).unwrap();
            let mut r = ReadOnlyBinaryStream::from_vec(w.into_data(), false);
            let read = r.read_normalized_f32().unwrap();
            assert!(
                (read - val).abs() < 0.001,
                "normalized float roundtrip failed for {}: got {}",
                val,
                read
            );
        }
    }

    // -----------------------------------------------------------------------
    // Full C++ bstream hex compatibility test
    // -----------------------------------------------------------------------
    #[test]
    fn test_write_matches_cpp_hex() {
        let expected_hex = "010203000400000005000000000000000100000000000018400000e0400800000009000000000000000a000b0c1a1c808080801000000010023137120000";

        let mut s = BinaryStream::new(false);
        s.write_u8(1).unwrap();
        s.write_u8(2).unwrap();
        s.write_u16(3).unwrap();
        s.write_u32(4).unwrap();
        s.write_u64(5).unwrap();
        s.write_bool(true).unwrap();
        s.write_f64(6.0).unwrap();
        s.write_f32(7.0).unwrap();
        s.write_i32(8).unwrap();
        s.write_i64(9).unwrap();
        s.write_i16(10).unwrap();
        s.write_unsigned_varint(11).unwrap();
        s.write_unsigned_varint64(12).unwrap();
        s.write_varint(13).unwrap();
        s.write_varint64(14).unwrap();
        s.write_normalized_f32(1.0).unwrap();
        s.write_i32_be(16).unwrap();
        s.write_string("17").unwrap();
        s.write_u24(18).unwrap();

        let got = hex::encode(s.into_data());
        assert_eq!(got, expected_hex, "hex output mismatch with C++ bstream");
    }
}
