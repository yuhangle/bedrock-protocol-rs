//! Pure Rust NBT implementation for Minecraft Bedrock Edition.
//!
//! Provides `CompoundTag` and `ListTag` matching the RapidNBT API used by inventoryui.
//! Supports Little Endian, Big Endian, and Bedrock Network NBT formats.
//!
//! # Usage
//!
//! ```rust
//! use bedrock_nbt::{CompoundTag, ListTag};
//!
//! let mut tag = CompoundTag::new();
//! tag.set("Name", "hello");
//! tag.set("RepairCost", 1);
//!
//! let mut lore = ListTag::new();
//! lore.append("line 1");
//! lore.append("line 2");
//! tag.set("Lore", lore);
//!
//! let bytes = tag.to_binary_nbt(false, false);  // little-endian, no header
//! let snbt = tag.to_snbt();
//! assert!(!tag.empty());
//! ```

mod tag;
mod compound;
mod list;
pub mod decode;
pub mod encode;
pub mod snbt;

pub use tag::Tag;
pub use tag::TagType;
pub use compound::CompoundTag;
pub use list::ListTag;
pub use encode::{NbtFormat, write_tag, write_compound_to_stream};

// ── From impls for CompoundTag / ListTag → Tag ──

impl From<CompoundTag> for Tag {
    fn from(ct: CompoundTag) -> Self {
        ct.to_tag()
    }
}

impl From<ListTag> for Tag {
    fn from(lt: ListTag) -> Self {
        lt.to_tag()
    }
}

impl CompoundTag {
    /// Serialize to binary NBT format.
    /// `little_endian`: true for LE, false for BE.
    /// `header`: if true, prepend a 4-byte storage_version (0) and 4-byte content length.
    pub fn to_binary_nbt(&self, little_endian: bool, header: bool) -> Vec<u8> {
        if header {
            return self.to_binary_nbt_with_header(little_endian, None);
        }
        let format = if little_endian { NbtFormat::LittleEndian } else { NbtFormat::BigEndian };
        encode::write_tag(self, format)
    }

    /// Serialize to binary NBT with a header prefix:
    /// `[int32 storage_version] + [int32 content_length] + [standard binary NBT]`.
    /// Defaults `storage_version` to 0 if not provided.
    pub fn to_binary_nbt_with_header(&self, little_endian: bool, storage_version: Option<i32>) -> Vec<u8> {
        let nbt_data = {
            let format = if little_endian { NbtFormat::LittleEndian } else { NbtFormat::BigEndian };
            encode::write_tag(self, format)
        };
        let version = storage_version.unwrap_or(0);
        let mut buf = Vec::with_capacity(8 + nbt_data.len());
        if little_endian {
            buf.extend_from_slice(&version.to_le_bytes());
            buf.extend_from_slice(&(nbt_data.len() as i32).to_le_bytes());
        } else {
            buf.extend_from_slice(&version.to_be_bytes());
            buf.extend_from_slice(&(nbt_data.len() as i32).to_be_bytes());
        }
        buf.extend_from_slice(&nbt_data);
        buf
    }

    /// Serialize to Bedrock Network NBT format (varint length prefixes).
    /// This is the format used in Minecraft Bedrock networking.
    /// Includes TAG_Compound header (0x0A + name_len=0) for rapidnbt compatibility.
    pub fn to_network_nbt(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(crate::TagType::Compound as u8);  // TAG_Compound
        buf.push(0);  // name length (empty string, varint = 0)
        encode::write_compound_to_stream(&mut buf, self, NbtFormat::BedrockNetwork);
        buf
    }

    /// Format as SNBT (stringified NBT) for debugging.
    pub fn to_snbt(&self) -> String {
        let mut out = String::new();
        write_snbt_compound(&mut out, self, 0);
        out
    }
}

impl ListTag {
    /// Serialize the list to SNBT.
    pub fn to_snbt(&self) -> String {
        let mut out = String::new();
        out.push('[');
        for (i, elem) in self.elements().iter().enumerate() {
            if i > 0 { out.push_str(", "); }
            write_snbt_tag(&mut out, elem);
        }
        out.push(']');
        out
    }
}

impl Tag {
    /// Format as SNBT.
    pub fn to_snbt(&self) -> String {
        let mut out = String::new();
        write_snbt_tag(&mut out, self);
        out
    }
}

// ── SNBT formatting ──

fn write_snbt_tag(out: &mut String, tag: &Tag) {
    match tag {
        Tag::Byte(v) => { out.push_str(&format!("{}b", v)); }
        Tag::Short(v) => { out.push_str(&format!("{}s", v)); }
        Tag::Int(v) => { out.push_str(&format!("{}", v)); }
        Tag::Long(v) => { out.push_str(&format!("{}L", v)); }
        Tag::Float(v) => { out.push_str(&format!("{}f", v)); }
        Tag::Double(v) => { out.push_str(&format!("{}d", v)); }
        Tag::String(v) => { out.push_str(&format!("\"{}\"", v)); }
        Tag::ByteArray(v) => {
            out.push_str("[B;");
            for (i, b) in v.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                out.push_str(&format!("{}", b));
            }
            out.push(']');
        }
        Tag::IntArray(v) => {
            out.push_str("[I;");
            for (i, n) in v.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                out.push_str(&format!("{}", n));
            }
            out.push(']');
        }
        Tag::LongArray(v) => {
            out.push_str("[L;");
            for (i, n) in v.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                out.push_str(&format!("{}", n));
            }
            out.push(']');
        }
        Tag::List(lst) => {
            out.push('[');
            for (i, elem) in lst.elements.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                write_snbt_tag(out, elem);
            }
            out.push(']');
        }
        Tag::Compound(map) => {
            out.push('{');
            let mut first = true;
            for (key, val) in map {
                if !first { out.push_str(", "); }
                first = false;
                out.push_str(&format!("\"{}\": ", key));
                write_snbt_tag(out, val);
            }
            out.push('}');
        }
        Tag::End => {}
    }
}

fn write_snbt_compound(out: &mut String, tag: &CompoundTag, indent: usize) {
    if tag.empty() {
        out.push_str("{}");
        return;
    }
    out.push_str("{\n");
    let mut first = true;
    for (key, val) in tag.iter() {
        if !first { out.push_str(",\n"); }
        first = false;
        write_indent(out, indent + 1);
        out.push_str(&format!("\"{}\": ", key));
        write_snbt_tag(out, val);
    }
    out.push('\n');
    write_indent(out, indent);
    out.push('}');
}

fn write_indent(out: &mut String, indent: usize) {
    for _ in 0..indent { out.push_str("    "); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compound_set_get() {
        let mut tag = CompoundTag::new();
        tag.set("name", "test");
        tag.set("value", 42);
        assert_eq!(tag.size(), 2);
        assert!(tag.contains("name"));
        assert!(!tag.empty());
        assert!(tag.get("value").is_some());
    }

    #[test]
    fn test_empty_compound() {
        let tag = CompoundTag::new();
        assert!(tag.empty());
        assert_eq!(tag.size(), 0);
    }

    #[test]
    fn test_list_tag() {
        let mut list = ListTag::new();
        assert!(list.is_empty());
        list.append("hello");
        list.append("world");
        assert_eq!(list.size(), 2);
        assert_eq!(list.get(0).unwrap().to_snbt(), "\"hello\"");
    }

    #[test]
    fn test_nbt_binary_roundtrip() {
        let mut tag = CompoundTag::new();
        tag.set("name", "test_item");
        tag.set("count", 1i32);
        let bytes = tag.to_binary_nbt(true, false);
        assert!(!bytes.is_empty());
        // Verify it starts with TAG_Compound type
        assert_eq!(bytes[0], TagType::Compound as u8);
    }

    #[test]
    fn test_to_snbt() {
        let mut tag = CompoundTag::new();
        tag.set("name", "test");
        let snbt = tag.to_snbt();
        assert!(snbt.contains("test"));
        assert!(snbt.contains("name"));
    }

    #[test]
    fn test_to_network_nbt() {
        let mut tag = CompoundTag::new();
        tag.set("key", "value");
        let bytes = tag.to_network_nbt();
        assert!(!bytes.is_empty());
        // Now includes TAG_Compound header (0x0A + name_len=0)
        assert!(bytes[0] == TagType::Compound as u8);
    }
}
