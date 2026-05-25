use crate::tag::{Tag, TagType};
use crate::CompoundTag;

/// NBT binary encoding format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NbtFormat {
    /// Standard little-endian NBT (u16 string length, i32 array length).
    LittleEndian,
    /// Little-endian with a header byte (used in some Bedrock contexts).
    LittleEndianWithHeader,
    /// Bedrock Network NBT (varint string/array length).
    BedrockNetwork,
    /// Standard big-endian NBT (Java Edition).
    BigEndian,
}

impl NbtFormat {
    fn is_little(&self) -> bool {
        matches!(self, NbtFormat::LittleEndian | NbtFormat::LittleEndianWithHeader | NbtFormat::BedrockNetwork)
    }

    fn has_header(&self) -> bool {
        matches!(self, NbtFormat::LittleEndianWithHeader)
    }
}

/// Write a CompoundTag to a `Vec<u8>` in the specified format.
pub fn write_tag(tag: &CompoundTag, format: NbtFormat) -> Vec<u8> {
    let mut buf = Vec::new();
    if format == NbtFormat::BedrockNetwork {
        // Bedrock Network format: write compound directly without root name
        write_compound_tag_contents(&mut buf, tag, format);
    } else if format.has_header() {
        // LittleEndianWithHeader / BigEndianWithHeader: prepend storage_version + content length
        let nbt_data = {
            let inner_format = if format.is_little() { NbtFormat::LittleEndian } else { NbtFormat::BigEndian };
            let mut inner = Vec::new();
            write_tag_value(&mut inner, &Tag::Compound(
                tag.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
            ), inner_format);
            inner
        };
        let version: i32 = 0;
        if format.is_little() {
            buf.extend_from_slice(&version.to_le_bytes());
            buf.extend_from_slice(&(nbt_data.len() as i32).to_le_bytes());
        } else {
            buf.extend_from_slice(&version.to_be_bytes());
            buf.extend_from_slice(&(nbt_data.len() as i32).to_be_bytes());
        }
        buf.extend_from_slice(&nbt_data);
    } else {
        // Standard format: write root tag with name (empty string for unnamed root)
        write_tag_value(&mut buf, &Tag::Compound(
            tag.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
        ), format);
    }
    buf
}

fn write_tag_value(buf: &mut Vec<u8>, tag: &Tag, format: NbtFormat) {
    match tag {
        Tag::Byte(v) => {
            buf.push(TagType::Byte as u8);
            buf.push(*v as u8);
        }
        Tag::Short(v) => {
            buf.push(TagType::Short as u8);
            if format == NbtFormat::BedrockNetwork {
                buf.extend_from_slice(&v.to_le_bytes());
            } else if format.is_little() {
                buf.extend_from_slice(&v.to_le_bytes());
            } else {
                buf.extend_from_slice(&v.to_be_bytes());
            }
        }
        Tag::Int(v) => {
            buf.push(TagType::Int as u8);
            if format == NbtFormat::BedrockNetwork {
                write_zigzag_i32(buf, *v);
            } else if format.is_little() {
                buf.extend_from_slice(&v.to_le_bytes());
            } else {
                buf.extend_from_slice(&v.to_be_bytes());
            }
        }
        Tag::Long(v) => {
            buf.push(TagType::Long as u8);
            if format == NbtFormat::BedrockNetwork {
                write_zigzag_i64(buf, *v);
            } else if format.is_little() {
                buf.extend_from_slice(&v.to_le_bytes());
            } else {
                buf.extend_from_slice(&v.to_be_bytes());
            }
        }
        Tag::Float(v) => {
            buf.push(TagType::Float as u8);
            if format.is_little() {
                buf.extend_from_slice(&v.to_le_bytes());
            } else {
                buf.extend_from_slice(&v.to_be_bytes());
            }
        }
        Tag::Double(v) => {
            buf.push(TagType::Double as u8);
            if format.is_little() {
                buf.extend_from_slice(&v.to_le_bytes());
            } else {
                buf.extend_from_slice(&v.to_be_bytes());
            }
        }
        Tag::ByteArray(v) => {
            buf.push(TagType::ByteArray as u8);
            write_length_u32(buf, v.len(), format);
            buf.extend_from_slice(v);
        }
        Tag::String(v) => {
            buf.push(TagType::String as u8);
            write_length(buf, v.len(), format);
            buf.extend_from_slice(v.as_bytes());
        }
        Tag::List(lst) => {
            buf.push(TagType::List as u8);
            buf.push(lst.element_type as u8);
            write_length_u32(buf, lst.elements.len(), format);
            for elem in &lst.elements {
                write_tag_payload(buf, elem, format);
            }
        }
        Tag::Compound(map) => {
            buf.push(TagType::Compound as u8);
            write_length(buf, 0, format);
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            for (key, val) in entries {
                buf.push(val.tag_type() as u8);
                write_name(buf, key, format);
                write_tag_payload(buf, val, format);
            }
            buf.push(TagType::End as u8);
        }
        Tag::IntArray(v) => {
            buf.push(TagType::IntArray as u8);
            write_length_u32(buf, v.len(), format);
            for elem in v {
                if format.is_little() {
                    buf.extend_from_slice(&elem.to_le_bytes());
                } else {
                    buf.extend_from_slice(&elem.to_be_bytes());
                }
            }
        }
        Tag::LongArray(v) => {
            buf.push(TagType::LongArray as u8);
            write_length_u32(buf, v.len(), format);
            for elem in v {
                if format.is_little() {
                    buf.extend_from_slice(&elem.to_le_bytes());
                } else {
                    buf.extend_from_slice(&elem.to_be_bytes());
                }
            }
        }
        Tag::End => {
            buf.push(TagType::End as u8);
        }
    }
}

/// Write a signed 32-bit integer as ZigZag varint (rapidnbt-compatible).
fn write_zigzag_i32(buf: &mut Vec<u8>, v: i32) {
    let mut val = ((v << 1) ^ (v >> 31)) as u32;
    loop {
        if val < 0x80 {
            buf.push(val as u8);
            break;
        } else {
            buf.push((val as u8) | 0x80);
            val >>= 7;
        }
    }
}

/// Write a signed 64-bit integer as ZigZag varint (rapidnbt-compatible).
fn write_zigzag_i64(buf: &mut Vec<u8>, v: i64) {
    let mut val = ((v << 1) ^ (v >> 63)) as u64;
    loop {
        if val < 0x80 {
            buf.push(val as u8);
            break;
        } else {
            buf.push((val as u8) | 0x80);
            val >>= 7;
        }
    }
}

fn write_length(buf: &mut Vec<u8>, len: usize, format: NbtFormat) {
    match format {
        NbtFormat::BedrockNetwork => {
            let mut v = len as u32;
            loop {
                if v < 0x80 { buf.push(v as u8); break; }
                else { buf.push((v as u8) | 0x80); v >>= 7; }
            }
        }
        _ => {
            debug_assert!(len <= u16::MAX as usize, "string length exceeds u16 range for non-network NBT format");
            if format.is_little() { buf.extend_from_slice(&(len as u16).to_le_bytes()); }
            else { buf.extend_from_slice(&(len as u16).to_be_bytes()); }
        }
    }
}

fn write_length_u32(buf: &mut Vec<u8>, len: usize, format: NbtFormat) {
    match format {
        NbtFormat::BedrockNetwork => {
            // ZigZag-encoded varint (rapidnbt compat: list count, array length use ZigZag)
            write_zigzag_i32(buf, len as i32);
        }
        _ => {
            if format.is_little() {
                buf.extend_from_slice(&(len as u32).to_le_bytes());
            } else {
                buf.extend_from_slice(&(len as u32).to_be_bytes());
            }
        }
    }
}

/// Recursively write a CompoundTag in "tag-in-stream" format (used by Bedrock protocol).
/// The compound tag is written as a sequence of named tag entries, terminated by TAG_End.
pub fn write_compound_to_stream(buf: &mut Vec<u8>, tag: &CompoundTag, format: NbtFormat) {
    for (name, val) in tag.iter_sorted() {
        buf.push(val.tag_type() as u8);
        write_name(buf, name, format);
        write_tag_payload(buf, val, format);
    }
    buf.push(TagType::End as u8);
}

fn write_name(buf: &mut Vec<u8>, name: &str, format: NbtFormat) {
    match format {
        NbtFormat::BedrockNetwork => {
            // Varint-prefixed name
            let bytes = name.as_bytes();
            let mut v = bytes.len() as u32;
            loop {
                if v < 0x80 {
                    buf.push(v as u8);
                    break;
                } else {
                    buf.push((v as u8) | 0x80);
                    v >>= 7;
                }
            }
            buf.extend_from_slice(bytes);
        }
        _ => {
            debug_assert!(name.len() <= u16::MAX as usize, "name length exceeds u16 range for non-network NBT format");
            if format.is_little() {
                buf.extend_from_slice(&(name.len() as u16).to_le_bytes());
            } else {
                buf.extend_from_slice(&(name.len() as u16).to_be_bytes());
            }
            buf.extend_from_slice(name.as_bytes());
        }
    }
}

fn write_tag_payload(buf: &mut Vec<u8>, tag: &Tag, format: NbtFormat) {
    match tag {
        Tag::Byte(v) => buf.push(*v as u8),
        Tag::Short(v) => {
            if format == NbtFormat::BedrockNetwork {
                buf.extend_from_slice(&v.to_le_bytes());
            } else if format.is_little() {
                buf.extend_from_slice(&v.to_le_bytes());
            } else {
                buf.extend_from_slice(&v.to_be_bytes());
            }
        }
        Tag::Int(v) => {
            if format == NbtFormat::BedrockNetwork {
                write_zigzag_i32(buf, *v);
            } else if format.is_little() {
                buf.extend_from_slice(&v.to_le_bytes());
            } else {
                buf.extend_from_slice(&v.to_be_bytes());
            }
        }
        Tag::Long(v) => {
            if format == NbtFormat::BedrockNetwork {
                write_zigzag_i64(buf, *v);
            } else if format.is_little() {
                buf.extend_from_slice(&v.to_le_bytes());
            } else {
                buf.extend_from_slice(&v.to_be_bytes());
            }
        }
        Tag::Float(v) => {
            if format.is_little() {
                buf.extend_from_slice(&v.to_le_bytes());
            } else {
                buf.extend_from_slice(&v.to_be_bytes());
            }
        }
        Tag::Double(v) => {
            if format.is_little() {
                buf.extend_from_slice(&v.to_le_bytes());
            } else {
                buf.extend_from_slice(&v.to_be_bytes());
            }
        }
        Tag::ByteArray(v) => {
            write_length_u32(buf, v.len(), format);
            buf.extend_from_slice(v);
        }
        Tag::String(v) => {
            write_length(buf, v.len(), format);
            buf.extend_from_slice(v.as_bytes());
        }
        Tag::List(lst) => {
            buf.push(lst.element_type as u8);
            write_length_u32(buf, lst.elements.len(), format);
            for elem in &lst.elements {
                write_tag_payload(buf, elem, format);
            }
        }
        Tag::Compound(map) => {
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            for (key, val) in entries {
                buf.push(val.tag_type() as u8);
                write_name(buf, key, format);
                write_tag_payload(buf, val, format);
            }
            buf.push(TagType::End as u8);
        }
        Tag::IntArray(v) => {
            write_length_u32(buf, v.len(), format);
            for elem in v {
                if format == NbtFormat::BedrockNetwork {
                    write_zigzag_i32(buf, *elem);
                } else if format.is_little() {
                    buf.extend_from_slice(&elem.to_le_bytes());
                } else {
                    buf.extend_from_slice(&elem.to_be_bytes());
                }
            }
        }
        Tag::LongArray(v) => {
            write_length_u32(buf, v.len(), format);
            for elem in v {
                if format == NbtFormat::BedrockNetwork {
                    write_zigzag_i64(buf, *elem);
                } else if format.is_little() {
                    buf.extend_from_slice(&elem.to_le_bytes());
                } else {
                    buf.extend_from_slice(&elem.to_be_bytes());
                }
            }
        }
        Tag::End => {}
    }
}

fn write_compound_tag_contents(buf: &mut Vec<u8>, tag: &CompoundTag, format: NbtFormat) {
    for (name, val) in tag.iter_sorted() {
        buf.push(val.tag_type() as u8);
        write_name(buf, name, format);
        write_tag_payload(buf, val, format);
    }
    buf.push(TagType::End as u8);
}
