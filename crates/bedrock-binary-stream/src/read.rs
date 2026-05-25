use bedrock_common::{BinaryStreamError, BedrockRead};

/// A read-only binary stream that wraps either a borrowed `&[u8]` or an owned `Vec<u8>`.
///
/// Tracks the current read position and an overflow flag. All read methods return
/// `Result` — no panics, even on overflow (the stream simply sets the overflow flag
/// and returns an error, while advancing the position as far as possible).
///
/// # Byte Order
///
/// Multi-byte integer types default to little-endian. Set `big_endian = true` at
/// construction for big-endian interpretation. Varints are endianness-independent.
pub struct ReadOnlyBinaryStream {
    buffer: Vec<u8>,
    position: usize,
    big_endian: bool,
    overflowed: bool,
}

impl ReadOnlyBinaryStream {
    /// Create a new stream from a borrowed byte slice.
    ///
    /// The data is copied into an owned buffer. To avoid the copy, use a library
    /// that wraps `&[u8]` directly. The `copy_buffer` semantics from the C++ bstream
    /// library are: when called with `copy_buffer = false` and a `bytes` input,
    /// the data must still be copied because Python `bytes` is immutable. In our Rust
    /// implementation we always own the buffer for simplicity.
    pub fn new(buffer: &[u8], big_endian: bool) -> Self {
        Self {
            buffer: buffer.to_vec(),
            position: 0,
            big_endian,
            overflowed: false,
        }
    }

    /// Create a new stream from an owned `Vec<u8>`.
    pub fn from_vec(buffer: Vec<u8>, big_endian: bool) -> Self {
        Self {
            buffer,
            position: 0,
            big_endian,
            overflowed: false,
        }
    }

    // -----------------------------------------------------------------------
    // Position / status
    // -----------------------------------------------------------------------

    /// Return the current read position.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Set the read position to an absolute offset.
    ///
    /// Returns an error if `pos` is beyond the buffer length, but also sets the
    /// position to the end of the buffer so subsequent reads will overflow.
    pub fn set_position(&mut self, pos: usize) -> Result<(), BinaryStreamError> {
        if pos > self.buffer.len() {
            self.position = self.buffer.len();
            self.overflowed = true;
            return Err(BinaryStreamError::Overflow {
                position: pos,
                size: self.buffer.len(),
            });
        }
        self.position = pos;
        Ok(())
    }

    /// Reset the read position to 0.
    pub fn reset_position(&mut self) {
        self.position = 0;
    }

    /// Return the total buffer size in bytes.
    pub fn size(&self) -> usize {
        self.buffer.len()
    }

    /// Return the number of bytes remaining (from current position to end).
    pub fn remaining(&self) -> usize {
        self.buffer.len().saturating_sub(self.position)
    }

    /// Check if there is unread data remaining.
    pub fn has_data_left(&self) -> bool {
        self.position < self.buffer.len()
    }

    /// Check whether an overflow has occurred at any point.
    pub fn has_overflowed(&self) -> bool {
        self.overflowed
    }

    /// Skip `count` bytes without reading them.
    pub fn ignore_bytes(&mut self, count: usize) -> Result<(), BinaryStreamError> {
        let new_pos = self.position + count;
        if new_pos > self.buffer.len() {
            self.position = self.buffer.len();
            self.overflowed = true;
            return Err(BinaryStreamError::Overflow {
                position: new_pos,
                size: self.buffer.len(),
            });
        }
        self.position = new_pos;
        Ok(())
    }

    /// Return a copy of the read-only portion of the buffer (not the whole buffer).
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    /// Return a copy of the entire buffer.
    pub fn copy_data(&self) -> Vec<u8> {
        self.buffer.clone()
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Read `n` bytes into a `Vec<u8>`, advancing the position.
    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>, BinaryStreamError> {
        if self.position + n > self.buffer.len() {
            self.overflowed = true;
            return Err(BinaryStreamError::Overflow {
                position: self.position + n,
                size: self.buffer.len(),
            });
        }
        let start = self.position;
        self.position += n;
        Ok(self.buffer[start..self.position].to_vec())
    }

    /// Read exactly one byte.
    fn read_one(&mut self) -> Result<u8, BinaryStreamError> {
        if self.position >= self.buffer.len() {
            self.overflowed = true;
            return Err(BinaryStreamError::Overflow {
                position: self.position,
                size: self.buffer.len(),
            });
        }
        let val = self.buffer[self.position];
        self.position += 1;
        Ok(val)
    }

    /// Read an unsigned varint and return the raw u64 value (before any ZigZag).
    fn read_raw_unsigned_varint(&mut self) -> Result<u64, BinaryStreamError> {
        let start = self.position;
        // Scan forward to find the end of the varint
        let mut end = start;
        while end < self.buffer.len() && self.buffer[end] & 0x80 != 0 {
            end += 1;
        }
        if end >= self.buffer.len() {
            self.position = self.buffer.len();
            self.overflowed = true;
            return Err(BinaryStreamError::InvalidData {
                description: "unexpected end of buffer while decoding varint",
            });
        }
        // Include the final byte (without continuation bit)
        end += 1;

        let slice = &self.buffer[start..end];
        self.position = end;

        // Decode as u64 (handles both 32-bit and 64-bit varints)
        let mut value: u64 = 0;
        for (i, &byte) in slice.iter().enumerate() {
            value |= ((byte & 0x7F) as u64) << (i * 7);
        }
        Ok(value)
    }
}

impl BedrockRead for ReadOnlyBinaryStream {
    fn read_bool(&mut self) -> Result<bool, BinaryStreamError> {
        self.read_one().map(|b| b != 0)
    }

    fn read_u8(&mut self) -> Result<u8, BinaryStreamError> {
        self.read_one()
    }

    fn read_i16(&mut self) -> Result<i16, BinaryStreamError> {
        let bytes = self.read_bytes(2)?;
        if self.big_endian {
            Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
        } else {
            Ok(i16::from_le_bytes([bytes[0], bytes[1]]))
        }
    }

    fn read_u16(&mut self) -> Result<u16, BinaryStreamError> {
        let bytes = self.read_bytes(2)?;
        if self.big_endian {
            Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
        } else {
            Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
        }
    }

    fn read_i32(&mut self) -> Result<i32, BinaryStreamError> {
        let bytes = self.read_bytes(4)?;
        if self.big_endian {
            Ok(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        } else {
            Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        }
    }

    fn read_u32(&mut self) -> Result<u32, BinaryStreamError> {
        let bytes = self.read_bytes(4)?;
        if self.big_endian {
            Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        } else {
            Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        }
    }

    fn read_i64(&mut self) -> Result<i64, BinaryStreamError> {
        let bytes = self.read_bytes(8)?;
        if self.big_endian {
            Ok(i64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))
        } else {
            Ok(i64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))
        }
    }

    fn read_u64(&mut self) -> Result<u64, BinaryStreamError> {
        let bytes = self.read_bytes(8)?;
        if self.big_endian {
            Ok(u64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))
        } else {
            Ok(u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))
        }
    }

    fn read_f32(&mut self) -> Result<f32, BinaryStreamError> {
        let bytes = self.read_bytes(4)?;
        if self.big_endian {
            Ok(f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        } else {
            Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        }
    }

    fn read_f64(&mut self) -> Result<f64, BinaryStreamError> {
        let bytes = self.read_bytes(8)?;
        if self.big_endian {
            Ok(f64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))
        } else {
            Ok(f64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))
        }
    }

    fn read_i32_be(&mut self) -> Result<i32, BinaryStreamError> {
        let bytes = self.read_bytes(4)?;
        Ok(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u32_be(&mut self) -> Result<u32, BinaryStreamError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u24(&mut self) -> Result<u32, BinaryStreamError> {
        let bytes = self.read_bytes(3)?;
        if self.big_endian {
            Ok(u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]))
        } else {
            Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]))
        }
    }

    fn read_varint(&mut self) -> Result<i32, BinaryStreamError> {
        let raw = self.read_raw_unsigned_varint()?;
        let value = (raw >> 1) as i32 ^ -((raw & 1) as i32);
        Ok(value)
    }

    fn read_varint64(&mut self) -> Result<i64, BinaryStreamError> {
        let raw = self.read_raw_unsigned_varint()?;
        let value = (raw >> 1) as i64 ^ -((raw & 1) as i64);
        Ok(value)
    }

    fn read_unsigned_varint(&mut self) -> Result<u32, BinaryStreamError> {
        let raw = self.read_raw_unsigned_varint()?;
        Ok(raw as u32)
    }

    fn read_unsigned_varint64(&mut self) -> Result<u64, BinaryStreamError> {
        self.read_raw_unsigned_varint()
    }

    fn read_normalized_f32(&mut self) -> Result<f32, BinaryStreamError> {
        let raw = self.read_varint64()?;
        Ok(raw as f32 / 2147483648.0f32)
    }

    fn read_string(&mut self) -> Result<String, BinaryStreamError> {
        let len = self.read_unsigned_varint()? as usize;
        let bytes = self.read_bytes(len)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    fn read_short_string(&mut self) -> Result<String, BinaryStreamError> {
        let len = self.read_u16()? as usize;
        let bytes = self.read_bytes(len)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    fn read_long_string(&mut self) -> Result<String, BinaryStreamError> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_bytes(len)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    fn read_raw_bytes(&mut self, len: usize) -> Result<Vec<u8>, BinaryStreamError> {
        self.read_bytes(len)
    }

    fn read_remaining(&mut self) -> Result<Vec<u8>, BinaryStreamError> {
        let remaining = self.remaining();
        self.read_bytes(remaining)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a stream from a hex string
    fn from_hex(hex: &str, big_endian: bool) -> ReadOnlyBinaryStream {
        let bytes = hex::decode(hex).expect("valid hex");
        ReadOnlyBinaryStream::from_vec(bytes, big_endian)
    }

    #[test]
    fn test_read_bool_true() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![1], false);
        assert!(s.read_bool().unwrap());
    }

    #[test]
    fn test_read_bool_false() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0], false);
        assert!(!s.read_bool().unwrap());
    }

    #[test]
    fn test_read_u8() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0xAB], false);
        assert_eq!(s.read_u8().unwrap(), 0xAB);
    }

    #[test]
    fn test_read_i16_le() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x34, 0x12], false);
        assert_eq!(s.read_i16().unwrap(), 0x1234);
    }

    #[test]
    fn test_read_i16_be() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x12, 0x34], true);
        assert_eq!(s.read_i16().unwrap(), 0x1234);
    }

    #[test]
    fn test_read_u32_le() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x78, 0x56, 0x34, 0x12], false);
        assert_eq!(s.read_u32().unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_u32_be() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x12, 0x34, 0x56, 0x78], true);
        assert_eq!(s.read_u32().unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_i64_le() {
        let mut s = ReadOnlyBinaryStream::from_vec(
            vec![0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01],
            false,
        );
        assert_eq!(s.read_i64().unwrap(), 0x0123456789ABCDEF);
    }

    #[test]
    fn test_read_f32_le() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x00, 0x00, 0xE0, 0x40], false);
        assert!((s.read_f32().unwrap() - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_read_f64_le() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x40], false);
        assert!((s.read_f64().unwrap() - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_read_i32_be() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x00, 0x00, 0x00, 0x10], false); // BE always
        assert_eq!(s.read_i32_be().unwrap(), 16);
    }

    #[test]
    fn test_read_u24_le() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x12, 0x00, 0x00], false);
        assert_eq!(s.read_u24().unwrap(), 0x12);
    }

    #[test]
    fn test_read_unsigned_varint_single_byte() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x0B], false);
        assert_eq!(s.read_unsigned_varint().unwrap(), 11);
    }

    #[test]
    fn test_read_varint_positive() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x1A], false); // ZigZag(13) = 26
        assert_eq!(s.read_varint().unwrap(), 13);
    }

    #[test]
    fn test_read_varint_negative() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x01], false); // ZigZag(-1) = 1
        assert_eq!(s.read_varint().unwrap(), -1);
    }

    #[test]
    fn test_read_string() {
        // uvarint(2) + "17"
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x02, 0x31, 0x37], false);
        assert_eq!(s.read_string().unwrap(), "17");
    }

    #[test]
    fn test_read_remaining() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x01, 0x02, 0x03], false);
        s.read_u8().unwrap();
        let rem = s.read_remaining().unwrap();
        assert_eq!(rem, vec![0x02, 0x03]);
    }

    #[test]
    fn test_position_and_size() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x01, 0x02, 0x03, 0x04], false);
        assert_eq!(s.size(), 4);
        assert_eq!(s.position(), 0);
        s.read_u8().unwrap();
        assert_eq!(s.position(), 1);
        s.set_position(3).unwrap();
        assert_eq!(s.position(), 3);
        s.reset_position();
        assert_eq!(s.position(), 0);
    }

    #[test]
    fn test_overflow() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x01], false);
        s.read_u8().unwrap();
        assert!(s.read_u8().is_err());
        assert!(s.has_overflowed());
    }

    #[test]
    fn test_set_position_beyond_end() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x01], false);
        assert!(s.set_position(100).is_err());
        assert!(s.has_overflowed());
    }

    #[test]
    fn test_has_data_left() {
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x01, 0x02], false);
        assert!(s.has_data_left());
        s.read_bytes(2).unwrap();
        assert!(!s.has_data_left());
    }

    #[test]
    fn test_normalized_float_roundtrip() {
        // write_normalized_float(1.0) → varint64(0x100000000) → hex: 80 80 80 80 10
        let mut s = ReadOnlyBinaryStream::from_vec(vec![0x80, 0x80, 0x80, 0x80, 0x10], false);
        let val = s.read_normalized_f32().unwrap();
        assert!((val - 1.0).abs() < 0.001);
    }

    // -----------------------------------------------------------------------
    // Comparison test: match C++ bstream hex from Python test
    // -----------------------------------------------------------------------
    #[test]
    fn test_cpp_bstream_compatibility() {
        let hex_str = "010203000400000005000000000000000100000000000018400000e0400800000009000000000000000a000b0c1a1c808080801000000010023137120000";
        let mut s = from_hex(hex_str, false);

        assert_eq!(s.read_u8().unwrap(), 1);
        assert_eq!(s.read_u8().unwrap(), 2);
        assert_eq!(s.read_u16().unwrap(), 3);
        assert_eq!(s.read_u32().unwrap(), 4);
        assert_eq!(s.read_u64().unwrap(), 5);
        assert_eq!(s.read_bool().unwrap(), true);
        assert!((s.read_f64().unwrap() - 6.0).abs() < f64::EPSILON);
        assert!((s.read_f32().unwrap() - 7.0).abs() < f32::EPSILON);
        assert_eq!(s.read_i32().unwrap(), 8);
        assert_eq!(s.read_i64().unwrap(), 9);
        assert_eq!(s.read_i16().unwrap(), 10);
        assert_eq!(s.read_unsigned_varint().unwrap(), 11);
        assert_eq!(s.read_unsigned_varint64().unwrap(), 12);
        assert_eq!(s.read_varint().unwrap(), 13);
        assert_eq!(s.read_varint64().unwrap(), 14);
        let nf = s.read_normalized_f32().unwrap();
        assert!((nf - 1.0).abs() < 0.001);
        assert_eq!(s.read_i32_be().unwrap(), 16);
        assert_eq!(s.read_string().unwrap(), "17");
        assert_eq!(s.read_u24().unwrap(), 18);
    }
}

// The hex crate is only needed for tests
#[cfg(test)]
extern crate hex;
