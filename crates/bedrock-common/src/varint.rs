use crate::BinaryStreamError;

/// Maximum number of bytes in a varint32 (5 bytes: max 10-bit value after zigzag).
pub const MAX_VARINT32_SIZE: usize = 5;
/// Maximum number of bytes in a varint64 (10 bytes: max 64-bit value after zigzag).
pub const MAX_VARINT64_SIZE: usize = 10;

// ---------------------------------------------------------------------------
// Unsigned varint32
// ---------------------------------------------------------------------------

/// Encode a `u32` as an unsigned varint (base-128 variable-length encoding).
///
/// Each byte uses 7 bits for data and the MSB as a continuation flag.
/// If the MSB is 1, more bytes follow; if 0, this is the last byte.
pub fn encode_unsigned_varint(mut value: u32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(MAX_VARINT32_SIZE);
    loop {
        if value < 0x80 {
            buf.push(value as u8);
            break;
        } else {
            buf.push((value as u8) | 0x80);
            value >>= 7;
        }
    }
    buf
}

/// Decode a `u32` unsigned varint from the front of a byte slice.
///
/// Returns the decoded value and the number of bytes consumed, or an error
/// if the buffer is empty or the varint is too long (>5 bytes).
pub fn decode_unsigned_varint(buf: &[u8]) -> Result<(u32, usize), BinaryStreamError> {
    let mut value: u32 = 0;
    let mut shift: u32 = 0;
    for (i, &byte) in buf.iter().enumerate() {
        if i >= MAX_VARINT32_SIZE {
            return Err(BinaryStreamError::InvalidData {
                description: "varint32 is too long (max 5 bytes)",
            });
        }
        value |= ((byte & 0x7F) as u32) << shift;
        if byte & 0x80 == 0 {
            return Ok((value, i + 1));
        }
        shift += 7;
    }
    Err(BinaryStreamError::InvalidData {
        description: "unexpected end of buffer while decoding varint32",
    })
}

/// Return the byte size of an encoded unsigned varint32 without writing it.
pub fn unsigned_varint_size(value: u32) -> usize {
    if value < 0x80 {
        return 1;
    }
    if value < 0x4000 {
        return 2;
    }
    if value < 0x200_000 {
        return 3;
    }
    if value < 0x10_000_000 {
        return 4;
    }
    5
}

// ---------------------------------------------------------------------------
// Signed varint32 (ZigZag encoding)
// ---------------------------------------------------------------------------

/// Encode an `i32` as a signed varint using ZigZag encoding.
///
/// ZigZag maps signed integers to unsigned integers so that small negative
/// values get small varint encodings: `encoded = (n << 1) ^ (n >> 31)`.
pub fn encode_varint32(value: i32) -> Vec<u8> {
    let unsigned = ((value << 1) ^ (value >> 31)) as u32;
    encode_unsigned_varint(unsigned)
}

/// Decode a signed varint32 (ZigZag encoded) from a byte slice.
pub fn decode_varint32(buf: &[u8]) -> Result<(i32, usize), BinaryStreamError> {
    let (unsigned, consumed) = decode_unsigned_varint(buf)?;
    let value = (unsigned >> 1) as i32 ^ (-((unsigned & 1) as i32));
    Ok((value, consumed))
}

/// Return the byte size of an encoded signed varint32 without writing it.
pub fn varint32_size(value: i32) -> usize {
    unsigned_varint_size(((value << 1) ^ (value >> 31)) as u32)
}

// ---------------------------------------------------------------------------
// Unsigned varint64
// ---------------------------------------------------------------------------

/// Encode a `u64` as an unsigned varint64.
pub fn encode_unsigned_varint64(mut value: u64) -> Vec<u8> {
    let mut buf = Vec::with_capacity(MAX_VARINT64_SIZE);
    loop {
        if value < 0x80 {
            buf.push(value as u8);
            break;
        } else {
            buf.push((value as u8) | 0x80);
            value >>= 7;
        }
    }
    buf
}

/// Decode a `u64` unsigned varint64 from a byte slice.
pub fn decode_unsigned_varint64(buf: &[u8]) -> Result<(u64, usize), BinaryStreamError> {
    let mut value: u64 = 0;
    let mut shift: u32 = 0;
    for (i, &byte) in buf.iter().enumerate() {
        if i >= MAX_VARINT64_SIZE {
            return Err(BinaryStreamError::InvalidData {
                description: "varint64 is too long (max 10 bytes)",
            });
        }
        value |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok((value, i + 1));
        }
        shift += 7;
    }
    Err(BinaryStreamError::InvalidData {
        description: "unexpected end of buffer while decoding varint64",
    })
}

/// Return the byte size of an encoded unsigned varint64 without writing it.
pub fn unsigned_varint64_size(value: u64) -> usize {
    if value < 0x80 {
        return 1;
    }
    if value < 0x4000 {
        return 2;
    }
    if value < 0x200_000 {
        return 3;
    }
    if value < 0x10_000_000 {
        return 4;
    }
    if value < 0x800_000_000 {
        return 5;
    }
    if value < 0x40_000_000_000 {
        return 6;
    }
    if value < 0x2_000_000_000_000 {
        return 7;
    }
    if value < 0x100_000_000_000_000 {
        return 8;
    }
    if value < 0x8_000_000_000_000_000 {
        return 9;
    }
    10
}

// ---------------------------------------------------------------------------
// Signed varint64 (ZigZag encoding)
// ---------------------------------------------------------------------------

/// Encode an `i64` as a signed varint64 using ZigZag encoding.
pub fn encode_varint64(value: i64) -> Vec<u8> {
    let unsigned = ((value << 1) ^ (value >> 63)) as u64;
    encode_unsigned_varint64(unsigned)
}

/// Decode a signed varint64 (ZigZag encoded) from a byte slice.
pub fn decode_varint64(buf: &[u8]) -> Result<(i64, usize), BinaryStreamError> {
    let (unsigned, consumed) = decode_unsigned_varint64(buf)?;
    let value = (unsigned >> 1) as i64 ^ (-((unsigned & 1) as i64));
    Ok((value, consumed))
}

/// Return the byte size of an encoded signed varint64 without writing it.
pub fn varint64_size(value: i64) -> usize {
    unsigned_varint64_size(((value << 1) ^ (value >> 63)) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Unsigned varint32 round-trip
    // -----------------------------------------------------------------------
    #[test]
    fn test_uvarint_roundtrip_zero() {
        let encoded = encode_unsigned_varint(0);
        assert_eq!(encoded, &[0x00]);
        let (decoded, _) = decode_unsigned_varint(&encoded).unwrap();
        assert_eq!(decoded, 0);
    }

    #[test]
    fn test_uvarint_roundtrip_one_byte_values() {
        for v in [0, 1, 0x7E, 0x7F] {
            let encoded = encode_unsigned_varint(v);
            let (decoded, _) = decode_unsigned_varint(&encoded).unwrap();
            assert_eq!(decoded, v);
        }
    }

    #[test]
    fn test_uvarint_roundtrip_two_byte_values() {
        for v in [0x80, 0x3FFF, 0x2000] {
            let encoded = encode_unsigned_varint(v);
            assert_eq!(encoded.len(), 2);
            let (decoded, _) = decode_unsigned_varint(&encoded).unwrap();
            assert_eq!(decoded, v);
        }
    }

    #[test]
    fn test_uvarint_roundtrip_max() {
        let v = u32::MAX;
        let encoded = encode_unsigned_varint(v);
        assert_eq!(encoded.len(), 5);
        let (decoded, _) = decode_unsigned_varint(&encoded).unwrap();
        assert_eq!(decoded, v);
    }

    #[test]
    fn test_uvarint_roundtrip_random() {
        let values = [
            42,
            127,
            128,
            16384,
            2097151,
            268435455,
            268435455,
            u32::MAX,
        ];
        for &v in &values {
            let encoded = encode_unsigned_varint(v);
            let (decoded, consumed) = decode_unsigned_varint(&encoded).unwrap();
            assert_eq!(decoded, v);
            assert_eq!(consumed, encoded.len());
        }
    }

    // -----------------------------------------------------------------------
    // Signed varint32 round-trip
    // -----------------------------------------------------------------------
    #[test]
    fn test_varint_roundtrip() {
        let values = [0, -1, 1, -2, 2, i32::MAX, i32::MIN, 42, -42, 127, -128];
        for &v in &values {
            let encoded = encode_varint32(v);
            let (decoded, _) = decode_varint32(&encoded).unwrap();
            assert_eq!(decoded, v);
        }
    }

    #[test]
    fn test_varint_size_consistency() {
        let values = [0, 1, -1, 127, -128, 16384, -16384, i32::MAX, i32::MIN];
        for &v in &values {
            let encoded = encode_varint32(v);
            assert_eq!(encoded.len(), varint32_size(v));
        }
    }

    // -----------------------------------------------------------------------
    // Unsigned varint64 round-trip
    // -----------------------------------------------------------------------
    #[test]
    fn test_uvarint64_roundtrip() {
        let values = [
            0u64,
            1,
            u64::MAX,
            u64::MAX >> 1,
            0xFFFFFFFFu64,
            0x7FFFFFFFFFFFFFFFu64,
        ];
        for &v in &values {
            let encoded = encode_unsigned_varint64(v);
            let (decoded, _) = decode_unsigned_varint64(&encoded).unwrap();
            assert_eq!(decoded, v);
        }
    }

    #[test]
    fn test_uvarint64_size_consistency() {
        let values = [0u64, 1, 127, 128, 16384, u64::MAX];
        for &v in &values {
            let encoded = encode_unsigned_varint64(v);
            assert_eq!(encoded.len(), unsigned_varint64_size(v));
        }
    }

    // -----------------------------------------------------------------------
    // Signed varint64 round-trip
    // -----------------------------------------------------------------------
    #[test]
    fn test_varint64_roundtrip() {
        let values = [
            0i64,
            -1,
            1,
            i64::MAX,
            i64::MIN,
            42,
            -42,
            127,
            -128,
            2147483647,
            -2147483648,
        ];
        for &v in &values {
            let encoded = encode_varint64(v);
            let (decoded, _) = decode_varint64(&encoded).unwrap();
            assert_eq!(decoded, v);
        }
    }

    // -----------------------------------------------------------------------
    // Error cases
    // -----------------------------------------------------------------------
    #[test]
    fn test_decode_uvarint_empty_buffer() {
        let result = decode_unsigned_varint(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_uvarint_too_long() {
        // 6 continuation bytes = invalid (max 5)
        let buf = [0x80, 0x80, 0x80, 0x80, 0x80, 0x01];
        let result = decode_unsigned_varint(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_uvarint64_too_long() {
        // 11 continuation bytes = invalid (max 10)
        let mut buf = [0x80u8; 11];
        buf[10] = 0x01;
        let result = decode_unsigned_varint64(&buf);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Property: decode(encode(x)) == x for all x in the test set
    // -----------------------------------------------------------------------
    #[test]
    fn test_property_varint_roundtrip() {
        for v in -1000i32..=1000i32 {
            let encoded = encode_varint32(v);
            let (decoded, _) = decode_varint32(&encoded).unwrap();
            assert_eq!(decoded, v);
        }
    }

    #[test]
    fn test_property_uvarint_roundtrip() {
        for v in 0u32..2000 {
            let encoded = encode_unsigned_varint(v);
            let (decoded, _) = decode_unsigned_varint(&encoded).unwrap();
            assert_eq!(decoded, v);
        }
    }
}
