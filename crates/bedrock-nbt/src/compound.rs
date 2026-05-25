use std::collections::HashMap;
use crate::tag::Tag;

/// A compound NBT tag — ordered map of named tags.
///
/// Matches the RapidNBT CompoundTag API used by inventoryui.
/// Values are auto-converted via `Tag::from_value()` / the `From<T>` impls.
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundTag {
    entries: Vec<(String, Tag)>,
}

impl CompoundTag {
    /// Create an empty compound tag.
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Create from a HashMap.
    pub fn from_map(map: HashMap<String, Tag>) -> Self {
        let entries: Vec<_> = map.into_iter().collect();
        // Preserve insertion order (as seen in the HashMap iteration)
        Self { entries }
    }

    /// Set a value by key. Auto-converts via `T: Into<Tag>`.
    pub fn set<T: Into<Tag>>(&mut self, key: &str, value: T) {
        let tag = value.into();
        if let Some(pos) = self.entries.iter().position(|(k, _)| k == key) {
            self.entries[pos].1 = tag;
        } else {
            self.entries.push((key.to_string(), tag));
        }
    }

    /// Get a reference to a tag by key. Returns `None` if not found.
    pub fn get(&self, key: &str) -> Option<&Tag> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Get a mutable reference to a tag by key.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Tag> {
        self.entries.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Check if a key exists.
    pub fn contains(&self, key: &str) -> bool {
        self.entries.iter().any(|(k, _)| k == key)
    }

    /// Check if the compound is empty.
    pub fn empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Number of entries.
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Remove a key.
    pub fn remove(&mut self, key: &str) -> bool {
        let pos = self.entries.iter().position(|(k, _)| k == key);
        if let Some(p) = pos {
            self.entries.remove(p);
            true
        } else {
            false
        }
    }

    /// Get all keys.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|(k, _)| k.as_str())
    }

    /// Iterate over all (key, tag) entries in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Tag)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Iterate over entries sorted alphabetically by key (rapidnbt compatible).
    pub fn iter_sorted(&self) -> Vec<(&str, &Tag)> {
        let mut items: Vec<_> = self.entries.iter().map(|(k, v)| (k.as_str(), v)).collect();
        items.sort_by(|a, b| a.0.cmp(b.0));
        items
    }

    /// Convert to internal Tag::Compound.
    pub fn to_tag(&self) -> Tag {
        let map: HashMap<String, Tag> = self.entries.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        Tag::Compound(map)
    }

    /// Convert to Tag with fields sorted alphabetically by key (rapidnbt compat).
    pub fn to_tag_sorted(&self) -> Tag {
        let mut entries: Vec<_> = self.entries.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let map: HashMap<String, Tag> = entries.into_iter().collect();
        Tag::Compound(map)
    }

    /// Get the inner entries (for encoding).
    pub fn entries(&self) -> &[(String, Tag)] {
        &self.entries
    }

    /// Rename a key. Returns true if the key existed and was renamed.
    pub fn rename(&mut self, old_key: &str, new_key: &str) -> bool {
        if let Some(pos) = self.entries.iter().position(|(k, _)| k == old_key) {
            let (_, v) = self.entries.remove(pos);
            if let Some(existing) = self.entries.iter_mut().find(|(k, _)| k == new_key) {
                existing.1 = v;
            } else {
                self.entries.push((new_key.to_string(), v));
            }
            true
        } else {
            false
        }
    }

    /// Insert a value only if the key does not already exist.
    /// Returns true if the value was inserted, false if the key already existed.
    pub fn put<T: Into<Tag>>(&mut self, key: &str, value: T) -> bool {
        if self.contains(key) {
            return false;
        }
        self.entries.push((key.to_string(), value.into()));
        true
    }

    /// Get a reference to a tag by key, panicking if the key is not present.
    /// For a non-panicking version, use `get()`.
    ///
    /// # Panics
    /// Panics if the key is not found in this compound.
    pub fn at(&self, key: &str) -> &Tag {
        self.get(key).unwrap_or_else(|| panic!("CompoundTag: key '{}' not found", key))
    }

    /// Parse a Network NBT CompoundTag from bytes (with TAG_Compound header).
    pub fn from_network_nbt(data: &[u8]) -> Result<(Self, usize), crate::decode::NbtDecodeError> {
        crate::decode::from_network_nbt(data)
    }

    /// Parse a Network NBT CompoundTag from bytes (without TAG_Compound header).
    pub fn from_network_nbt_contents(data: &[u8]) -> Result<(Self, usize), crate::decode::NbtDecodeError> {
        crate::decode::from_network_nbt_contents(data)
    }

    /// Parse a little-endian or big-endian binary NBT CompoundTag.
    /// The input should start with TAG_Compound header (0x0A + u16 name).
    pub fn from_binary_nbt(data: &[u8], little_endian: bool) -> Result<(Self, usize), crate::decode::NbtDecodeError> {
        crate::decode::from_binary_nbt(data, little_endian)
    }

    /// Parse binary NBT contents without TAG_Compound header.
    /// The input starts directly with field entries (TAG_End terminated).
    pub fn from_binary_nbt_contents(data: &[u8], little_endian: bool) -> Result<(Self, usize), crate::decode::NbtDecodeError> {
        crate::decode::from_binary_nbt_contents(data, little_endian)
    }

    /// Parse a binary NBT CompoundTag with a header prefix:
    /// `[int32 storage_version] + [int32 nbt_size] + [standard binary NBT]`.
    pub fn from_binary_nbt_with_header(data: &[u8], little_endian: bool) -> Result<(Self, usize), crate::decode::NbtDecodeError> {
        crate::decode::from_binary_nbt_with_header(data, little_endian)
    }

    /// Parse an SNBT string into a CompoundTag.
    ///
    /// The input should be a top-level compound tag: `{ key: value, ... }`.
    pub fn from_snbt(snbt: &str) -> Result<Self, crate::snbt::SnbtParseError> {
        crate::snbt::from_snbt(snbt)
    }

    /// Validate Network NBT byte sequence without fully decoding it.
    pub fn validate_network_nbt(data: &[u8]) -> bool {
        crate::decode::validate_network_nbt(data)
    }

    /// Validate binary NBT (LE/BE) byte sequence without fully decoding it.
    pub fn validate_binary_nbt(data: &[u8], little_endian: bool) -> bool {
        crate::decode::validate_binary_nbt(data, little_endian)
    }

    /// Merge entries from another CompoundTag into this one.
    ///
    /// When `merge_list` is true, list values with the same key are merged
    /// element-by-element instead of replaced.
    pub fn merge(&mut self, other: &CompoundTag, merge_list: bool) {
        for (key, val) in other.iter() {
            if merge_list {
                if let Some(Tag::List(existing_list)) = self.get(key) {
                    if let Tag::List(new_list) = val {
                        let mut merged = crate::ListTag::new();
                        for elem in &existing_list.elements {
                            merged.append(elem.clone());
                        }
                        for elem in &new_list.elements {
                            merged.append(elem.clone());
                        }
                        self.set(key, merged);
                        continue;
                    }
                }
            }
            self.set(key, val.clone());
        }
    }
}

impl Default for CompoundTag {
    fn default() -> Self { Self::new() }
}

// From impls are in lib.rs to handle cross-module references
