use crate::packet::PacketDefinition;
use crate::r#enum::EnumDefinition;
use crate::r#type::TypeDefinition;
use crate::SchemaError;
use serde::de;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

/// Load all packet definitions from a directory of JSON files.
pub fn load_packets(dir: &Path) -> Result<Vec<PacketDefinition>, SchemaError> {
    let mut packets = Vec::new();
    for entry in sorted_json_files(dir)? {
        let path = entry.path();
        let content = fs::read_to_string(&path)?;
        let packet: PacketDefinition = serde_json::from_str(&content).map_err(|e| {
            SchemaError::Json(de::Error::custom(format!("in {}: {}", path.display(), e)))
        })?;
        packets.push(packet);
    }
    Ok(packets)
}

/// Load all enum definitions from a directory of JSON files.
pub fn load_enums(dir: &Path) -> Result<Vec<EnumDefinition>, SchemaError> {
    let mut enums = Vec::new();
    for entry in sorted_json_files(dir)? {
        let path = entry.path();
        let content = fs::read_to_string(&path)?;
        let r#enum: EnumDefinition = serde_json::from_str(&content).map_err(|e| {
            SchemaError::Json(de::Error::custom(format!("in {}: {}", path.display(), e)))
        })?;
        enums.push(r#enum);
    }
    Ok(enums)
}

/// Load all type definitions from a directory of JSON files.
pub fn load_types(dir: &Path) -> Result<Vec<TypeDefinition>, SchemaError> {
    let mut types = Vec::new();
    for entry in sorted_json_files(dir)? {
        let path = entry.path();
        let content = fs::read_to_string(&path)?;
        let r#type: TypeDefinition = serde_json::from_str(&content).map_err(|e| {
            SchemaError::Json(de::Error::custom(format!("in {}: {}", path.display(), e)))
        })?;
        types.push(r#type);
    }
    Ok(types)
}

/// Get sorted list of JSON files in a directory.
fn sorted_json_files(dir: &Path) -> Result<Vec<fs::DirEntry>, SchemaError> {
    if !dir.is_dir() {
        return Err(SchemaError::InvalidDirectory {
            path: dir.display().to_string(),
            reason: "not a directory or does not exist".to_string(),
        });
    }

    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension() == Some(OsStr::new("json"))
        })
        .collect();

    entries.sort_by_key(|e| e.path());

    Ok(entries)
}
