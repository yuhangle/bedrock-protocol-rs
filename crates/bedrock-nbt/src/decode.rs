//! Network NBT deserialization for Minecraft Bedrock Edition.
//!
//! Parses Network NBT format where string lengths are varint-encoded.
use crate::{CompoundTag, Tag, TagType};
#[cfg(test)]
use crate::ListTag;
use std::fmt;

/// Error type for NBT decoding.
#[derive(Debug, Clone)]
pub enum NbtDecodeError {
    Overflow { position: usize, size: usize },
    InvalidData { description: &'static str },
}

impl fmt::Display for NbtDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow { position, size } => {
                write!(f, "buffer overflow at position {} (size: {})", position, size)
            }
            Self::InvalidData { description } => {
                write!(f, "invalid NBT data: {}", description)
            }
        }
    }
}

impl std::error::Error for NbtDecodeError {}

/// Parse a Network NBT CompoundTag from a byte slice.
///
/// The input should start with a TAG_Compound header (0x0A + varint name).
/// Returns the tag and the number of bytes consumed.
/// Maximum nesting depth for NBT compound tags to prevent stack overflow.
const MAX_NBT_DEPTH: u32 = 256;

pub fn from_network_nbt(data: &[u8]) -> Result<(CompoundTag, usize), NbtDecodeError> {
    if data.is_empty() {
        return Ok((CompoundTag::new(), 0));
    }
    let mut reader = NbtReader { data, pos: 0, depth: 0 };
    let tag_byte = reader.read_byte()?;
    if tag_byte == 0 {
        // Empty compound (just TAG_End) — return empty tag
        return Ok((CompoundTag::new(), 1));
    }
    if tag_byte != TagType::Compound as u8 {
        return Err(NbtDecodeError::InvalidData {
            description: "expected TAG_Compound",
        });
    }
    let _name = reader.read_name()?;
    let map = reader.read_compound_contents_map()?;
    Ok((CompoundTag::from_map(map), reader.pos))
}

/// Parse a Network NBT CompoundTag from a byte slice without header.
///
/// The input starts directly with field entries (TAG_End terminated),
/// without a leading TAG_Compound type byte or empty name.
pub fn from_network_nbt_contents(data: &[u8]) -> Result<(CompoundTag, usize), NbtDecodeError> {
    let mut reader = NbtReader { data, pos: 0, depth: 0 };
    let tag = reader.read_compound_contents()?;
    Ok((tag, reader.pos))
}

/// Parse a Little-Endian or Big-Endian binary NBT CompoundTag.
///
/// The input should start with a TAG_Compound header (0x0A + u16 name).
/// Returns the tag and the number of bytes consumed.
pub fn from_binary_nbt(data: &[u8], little_endian: bool) -> Result<(CompoundTag, usize), NbtDecodeError> {
    if data.is_empty() {
        return Ok((CompoundTag::new(), 0));
    }
    let mut reader = NbtReader { data, pos: 0, depth: 0 };
    let tag_byte = reader.read_byte()?;
    if tag_byte != TagType::Compound as u8 {
        return Err(NbtDecodeError::InvalidData {
            description: "expected TAG_Compound for root",
        });
    }
    let _name = reader.read_u16_string(little_endian)?;
    let map = reader.read_binary_compound_contents(little_endian)?;
    Ok((CompoundTag::from_map(map), reader.pos))
}

/// Parse a binary NBT CompoundTag from contents (without TAG_Compound header).
///
/// The input starts directly with field entries (TAG_End terminated),
/// without a leading TAG_Compound type byte or u16 name.
pub fn from_binary_nbt_contents(data: &[u8], little_endian: bool) -> Result<(CompoundTag, usize), NbtDecodeError> {
    let mut reader = NbtReader { data, pos: 0, depth: 0 };
    let map = reader.read_binary_compound_contents(little_endian)?;
    Ok((CompoundTag::from_map(map), reader.pos))
}

/// Validate that `data` is a valid Network NBT byte sequence.
pub fn validate_network_nbt(data: &[u8]) -> bool {
    from_network_nbt(data).is_ok()
}

/// Validate that `data` is a valid binary NBT (LE/BE) byte sequence.
pub fn validate_binary_nbt(data: &[u8], little_endian: bool) -> bool {
    from_binary_nbt(data, little_endian).is_ok()
}

/// Parse a binary NBT CompoundTag with a header prefix.
///
/// Format: `[int32 storage_version] + [int32 nbt_size] + [standard binary NBT]`.
pub fn from_binary_nbt_with_header(data: &[u8], little_endian: bool) -> Result<(CompoundTag, usize), NbtDecodeError> {
    if data.is_empty() {
        return Ok((CompoundTag::new(), 0));
    }
    let mut reader = NbtReader { data, pos: 0, depth: 0 };
    let _storage_version = reader.read_i32_ne(little_endian)?;
    let _nbt_size = reader.read_i32_ne(little_endian)? as usize;
    from_binary_nbt(&data[reader.pos..], little_endian)
        .map(|(tag, consumed)| (tag, reader.pos + consumed))
}

struct NbtReader<'a> {
    data: &'a [u8],
    pos: usize,
    depth: u32,
}

impl NbtReader<'_> {
    fn read_byte(&mut self) -> Result<u8, NbtDecodeError> {
        if self.pos >= self.data.len() {
            return Err(NbtDecodeError::Overflow {
                position: self.pos,
                size: self.data.len(),
            });
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn read_varint(&mut self) -> Result<u32, NbtDecodeError> {
        let mut val = 0u32;
        let mut shift = 0;
        loop {
            let b = self.read_byte()?;
            val |= ((b & 0x7f) as u32) << shift;
            shift += 7;
            if b & 0x80 == 0 {
                break;
            }
            if shift >= 35 {
                return Err(NbtDecodeError::InvalidData {
                    description: "varint too long",
                });
            }
        }
        Ok(val)
    }

    /// Read a signed 32-bit LE (for float/double).
    fn read_i32_le(&mut self) -> Result<i32, NbtDecodeError> {
        let mut val = 0i32;
        for i in 0..4 {
            let b = self.read_byte()? as i32;
            val |= b << (i * 8);
        }
        Ok(val)
    }

    /// Read a signed 64-bit LE (for double).
    fn read_i64_le(&mut self) -> Result<i64, NbtDecodeError> {
        let mut val = 0i64;
        for i in 0..8 {
            let b = self.read_byte()? as i64;
            val |= b << (i * 8);
        }
        Ok(val)
    }

    /// Read a ZigZag-encoded signed varint32 (rapidnbt Int/TAG_Int).
    fn read_zigzag_i32(&mut self) -> Result<i32, NbtDecodeError> {
        let val = self.read_varint()?;
        Ok(((val as i32) >> 1) ^ (-((val & 1) as i32)))
    }

    /// Read a ZigZag-encoded signed varint64 (rapidnbt Long/TAG_Long).
    fn read_zigzag_i64(&mut self) -> Result<i64, NbtDecodeError> {
        let val = self.read_u64_varint()?;
        Ok(((val as i64) >> 1) ^ (-((val & 1) as i64)))
    }

    fn read_u64_varint(&mut self) -> Result<u64, NbtDecodeError> {
        let mut val = 0u64;
        let mut shift = 0;
        loop {
            let b = self.read_byte()?;
            val |= ((b & 0x7f) as u64) << shift;
            shift += 7;
            if b & 0x80 == 0 { break; }
            if shift >= 70 {
                return Err(NbtDecodeError::InvalidData {
                    description: "varint64 too long",
                });
            }
        }
        Ok(val)
    }

    fn read_f32(&mut self) -> Result<f32, NbtDecodeError> {
        let bits = self.read_i32_le()?;
        Ok(f32::from_le_bytes(bits.to_le_bytes()))
    }

    fn read_f64(&mut self) -> Result<f64, NbtDecodeError> {
        let bits = self.read_i64_le()?;
        Ok(f64::from_le_bytes(bits.to_le_bytes()))
    }

    fn read_string(&mut self) -> Result<String, NbtDecodeError> {
        let len = self.read_varint()? as usize;
        if self.pos + len > self.data.len() {
            return Err(NbtDecodeError::Overflow {
                position: self.pos + len,
                size: self.data.len(),
            });
        }
        // Use lossy UTF-8 to match rapidnbt behavior (some items have non-UTF-8 field names/values)
        let s = String::from_utf8_lossy(&self.data[self.pos..self.pos + len]).into_owned();
        self.pos += len;
        Ok(s)
    }

    fn read_name(&mut self) -> Result<String, NbtDecodeError> {
        self.read_string()
    }

    /// Read a tag value based on tag type (no type byte, no name — just the value).
    fn read_tag_value(&mut self, tag_type: u8) -> Result<Tag, NbtDecodeError> {
        match tag_type {
            0 => Ok(Tag::End),
            1 => Ok(Tag::Byte(self.read_byte()? as i8)),
            2 => {
                let lo = self.read_byte()? as i16;
                let hi = self.read_byte()? as i16;
                Ok(Tag::Short(lo | (hi << 8)))
            }
            3 => Ok(Tag::Int(self.read_zigzag_i32()?)),
            4 => Ok(Tag::Long(self.read_zigzag_i64()?)),
            5 => Ok(Tag::Float(self.read_f32()?)),
            6 => Ok(Tag::Double(self.read_f64()?)),
            7 => {
                let len = self.read_zigzag_i32()? as usize;
                if self.pos + len > self.data.len() {
                    return Err(NbtDecodeError::Overflow {
                        position: self.pos + len,
                        size: self.data.len(),
                    });
                }
                let bytes = self.data[self.pos..self.pos + len].to_vec();
                self.pos += len;
                Ok(Tag::ByteArray(bytes))
            }
            8 => Ok(Tag::String(self.read_string()?)),
            9 => {
                let elem_type = self.read_byte()?;
                let count = self.read_zigzag_i32()?;
                let mut elements = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    elements.push(self.read_tag_value(elem_type)?);
                }
                Ok(Tag::List(crate::tag::ListTagValue {
                    element_type: TagType::from_u8(elem_type).unwrap_or(TagType::End),
                    elements,
                }))
            }
            10 => {
                let map = self.read_compound_contents_map()?;
                Ok(Tag::Compound(map))
            }
            11 => {
                let len = self.read_zigzag_i32()? as usize;
                let mut vals = Vec::with_capacity(len);
                for _ in 0..len {
                    vals.push(self.read_zigzag_i32()?);
                }
                Ok(Tag::IntArray(vals))
            }
            12 => {
                let len = self.read_zigzag_i32()? as usize;
                let mut vals = Vec::with_capacity(len);
                for _ in 0..len {
                    vals.push(self.read_zigzag_i64()?);
                }
                Ok(Tag::LongArray(vals))
            }
            _ => Err(NbtDecodeError::InvalidData {
                description: "unknown NBT tag type",
            }),
        }
    }

    /// Read compound fields until TAG_End, return the HashMap.
    fn read_compound_contents_map(&mut self) -> Result<std::collections::HashMap<String, Tag>, NbtDecodeError> {
        if self.depth >= MAX_NBT_DEPTH {
            return Err(NbtDecodeError::InvalidData {
                description: "NBT nesting depth exceeds maximum",
            });
        }
        self.depth += 1;
        let mut map = std::collections::HashMap::new();
        loop {
            let tag_type = self.read_byte()?;
            if tag_type == 0 {
                break; // TAG_End
            }
            let name = self.read_name()?;
            let tag = self.read_tag_value(tag_type)?;
            map.insert(name, tag);
        }
        self.depth -= 1;
        Ok(map)
    }

    /// Read a CompoundTag from content (no TAG_Compound header, start with fields).
    fn read_compound_contents(&mut self) -> Result<CompoundTag, NbtDecodeError> {
        let map = self.read_compound_contents_map()?;
        Ok(CompoundTag::from_map(map))
    }

    // ── Binary LE/BE format reader methods ──

    /// Read a u16-prefixed string (standard LE/BE NBT format).
    fn read_u16_string(&mut self, little_endian: bool) -> Result<String, NbtDecodeError> {
        let lo = self.read_byte()? as u16;
        let hi = self.read_byte()? as u16;
        let len = if little_endian { lo | (hi << 8) } else { (lo << 8) | hi } as usize;
        if self.pos + len > self.data.len() {
            return Err(NbtDecodeError::Overflow {
                position: self.pos + len,
                size: self.data.len(),
            });
        }
        let s = String::from_utf8_lossy(&self.data[self.pos..self.pos + len]).into_owned();
        self.pos += len;
        Ok(s)
    }

    /// Read a 4-byte native-endian i32.
    fn read_i32_ne(&mut self, little_endian: bool) -> Result<i32, NbtDecodeError> {
        let mut buf = [0u8; 4];
        for i in 0..4 { buf[i] = self.read_byte()?; }
        if little_endian { Ok(i32::from_le_bytes(buf)) } else { Ok(i32::from_be_bytes(buf)) }
    }

    /// Read a tag value in binary LE/BE format (given tag type byte, no type byte read here).
    fn read_binary_tag_value(&mut self, tag_type: u8, little_endian: bool) -> Result<Tag, NbtDecodeError> {
        match tag_type {
            0 => Ok(Tag::End),
            1 => Ok(Tag::Byte(self.read_byte()? as i8)),
            2 => {
                let lo = self.read_byte()? as i16;
                let hi = self.read_byte()? as i16;
                if little_endian { Ok(Tag::Short(lo | (hi << 8))) } else { Ok(Tag::Short((lo << 8) | hi)) }
            }
            3 => Ok(Tag::Int(self.read_i32_ne(little_endian)?)),
            4 => {
                let mut buf = [0u8; 8];
                for i in 0..8 { buf[i] = self.read_byte()?; }
                if little_endian { Ok(Tag::Long(i64::from_le_bytes(buf))) } else { Ok(Tag::Long(i64::from_be_bytes(buf))) }
            }
            5 => {
                let mut buf = [0u8; 4];
                for i in 0..4 { buf[i] = self.read_byte()?; }
                if little_endian { Ok(Tag::Float(f32::from_le_bytes(buf))) } else { Ok(Tag::Float(f32::from_be_bytes(buf))) }
            }
            6 => {
                let mut buf = [0u8; 8];
                for i in 0..8 { buf[i] = self.read_byte()?; }
                if little_endian { Ok(Tag::Double(f64::from_le_bytes(buf))) } else { Ok(Tag::Double(f64::from_be_bytes(buf))) }
            }
            7 => {
                let len = self.read_i32_ne(little_endian)? as usize;
                if self.pos + len > self.data.len() {
                    return Err(NbtDecodeError::Overflow {
                        position: self.pos + len,
                        size: self.data.len(),
                    });
                }
                let bytes = self.data[self.pos..self.pos + len].to_vec();
                self.pos += len;
                Ok(Tag::ByteArray(bytes))
            }
            8 => Ok(Tag::String(self.read_u16_string(little_endian)?)),
            9 => {
                let elem_type = self.read_byte()?;
                let count = self.read_i32_ne(little_endian)?;
                let mut elements = Vec::with_capacity(count.max(0) as usize);
                for _ in 0..count.max(0) {
                    elements.push(self.read_binary_tag_value(elem_type, little_endian)?);
                }
                Ok(Tag::List(crate::tag::ListTagValue {
                    element_type: TagType::from_u8(elem_type).unwrap_or(TagType::End),
                    elements,
                }))
            }
            10 => {
                let map = self.read_binary_compound_contents(little_endian)?;
                Ok(Tag::Compound(map))
            }
            11 => {
                let len = self.read_i32_ne(little_endian)? as usize;
                let mut vals = Vec::with_capacity(len);
                for _ in 0..len {
                    vals.push(self.read_i32_ne(little_endian)?);
                }
                Ok(Tag::IntArray(vals))
            }
            12 => {
                let len = self.read_i32_ne(little_endian)? as usize;
                let mut buf = [0u8; 8];
                let mut vals = Vec::with_capacity(len);
                for _ in 0..len {
                    for i in 0..8 { buf[i] = self.read_byte()?; }
                    if little_endian { vals.push(i64::from_le_bytes(buf)); } else { vals.push(i64::from_be_bytes(buf)); }
                }
                Ok(Tag::LongArray(vals))
            }
            _ => Err(NbtDecodeError::InvalidData {
                description: "unknown NBT tag type",
            }),
        }
    }

    /// Read compound fields in binary LE/BE format until TAG_End.
    fn read_binary_compound_contents(&mut self, little_endian: bool) -> Result<std::collections::HashMap<String, Tag>, NbtDecodeError> {
        if self.depth >= MAX_NBT_DEPTH {
            return Err(NbtDecodeError::InvalidData {
                description: "NBT nesting depth exceeds maximum",
            });
        }
        self.depth += 1;
        let mut map = std::collections::HashMap::new();
        loop {
            let tag_type = self.read_byte()?;
            if tag_type == 0 { break; }
            let name = self.read_u16_string(little_endian)?;
            let tag = self.read_binary_tag_value(tag_type, little_endian)?;
            map.insert(name, tag);
        }
        self.depth -= 1;
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_simple() {
        let mut tag = CompoundTag::new();
        tag.set("name", "test_value");
        tag.set("count", 42i32);

        let bytes = tag.to_network_nbt();
        eprintln!("bytes({}): {:02x?}", bytes.len(), bytes);
        let (parsed, _) = from_network_nbt(&bytes).unwrap();

        assert_eq!(parsed.get("name").and_then(|t| t.as_str()), Some("test_value"));
        assert_eq!(parsed.get("count").and_then(|t| t.as_i32()), Some(42));
    }

    #[test]
    fn test_roundtrip_nested() {
        let mut outer = CompoundTag::new();
        let mut inner = CompoundTag::new();
        inner.set("x", 10i32);
        inner.set("y", 20i32);
        outer.set("pos", inner);

        let mut list = ListTag::new();
        list.append("a");
        list.append("b");
        outer.set("items", list);

        let bytes = outer.to_network_nbt();
        eprintln!("nested bytes({}): {:02x?}", bytes.len(), bytes);
        let (parsed, _) = from_network_nbt(&bytes).unwrap();

        eprintln!("parsed keys: {:?}", parsed.keys().collect::<Vec<_>>());
        let pos = parsed.get("pos").and_then(|t| t.as_compound()).unwrap();
        eprintln!("pos keys: {:?}", pos.keys().collect::<Vec<_>>());
        assert_eq!(pos.get("x").and_then(|t| t.as_i32()), Some(10));
        let items = parsed.get("items").and_then(|t| t.as_list_value()).unwrap();
        assert_eq!(items.elements.len(), 2);
    }

    #[test]
    fn test_empty_compound() {
        let tag = CompoundTag::new();
        let bytes = tag.to_network_nbt();
        let (parsed, _) = from_network_nbt(&bytes).unwrap();
        assert!(parsed.empty());
    }

    /// Compatibility test matching RapidNBT `tests/network.py` serialization output.
    ///
    /// Expected bytes (from the Python test, without the leading 0x17 stream byte):
    /// `0a 00 01 08 62 79 74 65 5f 74 61 67 72 03 07 69 6e 74 5f 74 61 67 a4 fd 0d
    ///  02 09 73 68 6f 72 74 5f 74 61 67 bc 4a 08 0a 73 74 72 69 6e 67 5f 74 61 67
    ///  0b 54 65 73 74 20 53 74 72 69 6e 67 00`
    #[test]
    fn test_rapidnbt_compat_serialize() {
        let mut tag = CompoundTag::new();
        tag.set("string_tag", "Test String");
        tag.set("byte_tag", 114i8);
        tag.set("short_tag", 19132i16);
        tag.set("int_tag", 114514i32);

        let bytes = tag.to_network_nbt();

        let expected: Vec<u8> = vec![
            0x0a, 0x00, // TAG_Compound + name length 0
            0x01, 0x08, // TAG_Byte + name length 8
            b'b', b'y', b't', b'e', b'_', b't', b'a', b'g', // "byte_tag"
            0x72, // value 114
            0x03, 0x07, // TAG_Int + name length 7
            b'i', b'n', b't', b'_', b't', b'a', b'g', // "int_tag"
            0xa4, 0xfd, 0x0d, // ZigZag varint(114514)
            0x02, 0x09, // TAG_Short + name length 9
            b's', b'h', b'o', b'r', b't', b'_', b't', b'a', b'g', // "short_tag"
            0xbc, 0x4a, // LE 16-bit (19132)
            0x08, 0x0a, // TAG_String + name length 10
            b's', b't', b'r', b'i', b'n', b'g', b'_', b't', b'a', b'g', // "string_tag"
            0x0b, // string value length 11
            b'T', b'e', b's', b't', b' ', b'S', b't', b'r', b'i', b'n', b'g', // "Test String"
            0x00, // TAG_End
        ];

        assert_eq!(bytes, expected, "Network NBT serialization does not match RapidNBT output");
    }

    #[test]
    fn test_rapidnbt_compat_roundtrip() {
        let mut tag = CompoundTag::new();
        tag.set("string_tag", "Test String");
        tag.set("byte_tag", 114i8);
        tag.set("short_tag", 19132i16);
        tag.set("int_tag", 114514i32);

        let bytes = tag.to_network_nbt();
        let (parsed, consumed) = from_network_nbt(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());

        assert_eq!(parsed.at("byte_tag"), &Tag::Byte(114));
        assert_eq!(parsed.at("short_tag"), &Tag::Short(19132));
        assert_eq!(parsed.at("int_tag"), &Tag::Int(114514));
        assert_eq!(parsed.at("string_tag").as_str(), Some("Test String"));
    }
}
