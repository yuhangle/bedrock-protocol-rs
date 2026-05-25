use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Container for a single protocol version's raw JSON data.
///
/// This is the format used for compile-time embedding.
/// build.rs reads the JSON files, packages them into this struct,
/// serializes to JSON, and embeds via `include_str!()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedVersion {
    /// The network protocol version number (e.g., 975 for r26_u2).
    pub network_version: u32,

    /// The human-readable branch name (e.g., "r26_u2").
    pub branch_name: String,

    /// The Minecraft version string (e.g., "1.26.20.28").
    pub minecraft_version: String,

    /// Packet definitions: filename → JSON content.
    pub packets: HashMap<String, String>,

    /// Enum definitions: filename → JSON content.
    pub enums: HashMap<String, String>,

    /// Type definitions: filename → JSON content.
    pub types: HashMap<String, String>,
}

impl EmbeddedVersion {
    /// Build an EmbeddedVersion from a protocol-docs directory path.
    /// Reads all JSON files from the packets/, enums/, types/ subdirectories.
    pub fn from_directory(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        use std::fs;

        let readme = fs::read_to_string(path.join("README.md")).ok();

        // Parse network version from README (format: "- **Network Version:** 975")
        let network_version = readme
            .as_ref()
            .and_then(|r| {
                r.lines()
                    .find(|l| l.contains("Network Version"))
                    .and_then(|l| {
                        // Split by ':' and take the last segment (the number after "**")
                        l.split(':').last()
                            .map(|s| s.trim().trim_matches('*').trim())
                    })
                    .and_then(|s| s.parse::<u32>().ok())
            })
            .unwrap_or(0);

        let branch_name = readme
            .as_ref()
            .and_then(|r| {
                r.lines()
                    .find(|l| l.contains("r26") || l.contains("r21"))
                    .and_then(|l| {
                        l.split('(')
                            .nth(1)
                            .or_else(|| l.split(':').nth(1))
                    })
                    .map(|s| s.trim().trim_end_matches(')').to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        let minecraft_version = readme
            .as_ref()
            .and_then(|r| {
                r.lines()
                    .find(|l| l.contains("Minecraft Version:"))
                    .and_then(|l| l.split(':').nth(1))
                    .map(|s| s.trim().trim_matches('*').trim().to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        let packets = read_json_map(&path.join("packets"))?;
        let enums = read_json_map(&path.join("enums"))?;
        let types = read_json_map(&path.join("types"))?;

        Ok(Self {
            network_version,
            branch_name,
            minecraft_version,
            packets,
            enums,
            types,
        })
    }

    /// Load a Schema from this embedded version data.
    pub fn to_schema(&self) -> Result<crate::Schema, crate::SchemaError> {
        let mut packets = Vec::new();
        for (_name, content) in &self.packets {
            let p: crate::packet::PacketDefinition = serde_json::from_str(content)?;
            packets.push(p);
        }

        let mut enums = Vec::new();
        for (_name, content) in &self.enums {
            let e: crate::r#enum::EnumDefinition = serde_json::from_str(content)?;
            enums.push(e);
        }

        let mut types = Vec::new();
        for (_name, content) in &self.types {
            let t: crate::r#type::TypeDefinition = serde_json::from_str(content)?;
            types.push(t);
        }

        Ok(crate::Schema::from_lists(packets, enums, types))
    }
}

fn read_json_map(dir: &std::path::Path) -> Result<HashMap<String, String>, std::io::Error> {
    let mut map = HashMap::new();
    if !dir.is_dir() {
        return Ok(map);
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let content = std::fs::read_to_string(&path)?;
            map.insert(name, content);
        }
    }
    Ok(map)
}
