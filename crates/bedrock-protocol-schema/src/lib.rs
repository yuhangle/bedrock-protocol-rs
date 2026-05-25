//! Parse protocol-docs JSON schemas for Minecraft Bedrock Edition.
//!
//! Reads the JSON protocol documentation (packets, enums, types) and provides
//! a typed query API for accessing packet definitions by ID, enums by name,
//! and type definitions by name.

pub mod field;
pub mod packet;
pub mod r#enum;
pub mod r#type;
pub mod embed;
pub mod registry;

mod parse;

pub use field::FieldDefinition;
pub use packet::PacketDefinition;
pub use r#enum::{EnumDefinition, EnumValue};
pub use r#type::TypeDefinition;

use std::collections::HashMap;
use std::path::Path;

/// The complete protocol schema, loaded from the protocol-docs JSON files.
pub struct Schema {
    pub packets: Vec<PacketDefinition>,
    pub enums: Vec<EnumDefinition>,
    pub types: Vec<TypeDefinition>,
    packet_map: HashMap<u32, usize>,
    packet_name_map: HashMap<String, usize>,
    enum_map: HashMap<String, usize>,
    type_map: HashMap<String, usize>,
}

impl Schema {
    /// Load all protocol documentation from a `protocol-docs` directory.
    pub fn load_all(base_path: &Path) -> Result<Self, SchemaError> {
        let packets_dir = base_path.join("packets");
        let enums_dir = base_path.join("enums");
        let types_dir = base_path.join("types");

        let packets = parse::load_packets(&packets_dir)?;
        let enums = parse::load_enums(&enums_dir)?;
        let types = parse::load_types(&types_dir)?;

        Ok(Self::from_lists(packets, enums, types))
    }

    /// Construct a Schema from pre-parsed lists (no filesystem access).
    /// Used by EmbeddedVersion::to_schema() for compile-time embedded protocol data.
    pub fn from_lists(
        packets: Vec<PacketDefinition>,
        enums: Vec<EnumDefinition>,
        types: Vec<TypeDefinition>,
    ) -> Self {
        let mut packet_map = HashMap::new();
        let mut packet_name_map = HashMap::new();
        for (i, p) in packets.iter().enumerate() {
            packet_map.insert(p.id, i);
            packet_name_map.insert(p.name.clone(), i);
        }

        let mut enum_map = HashMap::new();
        for (i, e) in enums.iter().enumerate() {
            enum_map.insert(e.name.clone(), i);
        }

        let mut type_map = HashMap::new();
        for (i, t) in types.iter().enumerate() {
            type_map.insert(t.name.clone(), i);
        }

        Self {
            packets,
            enums,
            types,
            packet_map,
            packet_name_map,
            enum_map,
            type_map,
        }
    }

    pub fn get_packet_by_id(&self, id: u32) -> Option<&PacketDefinition> {
        self.packet_map.get(&id).map(|&i| &self.packets[i])
    }

    pub fn get_packet_by_name(&self, name: &str) -> Option<&PacketDefinition> {
        self.packet_name_map.get(name).map(|&i| &self.packets[i])
    }

    pub fn get_enum_by_name(&self, name: &str) -> Option<&EnumDefinition> {
        self.enum_map.get(name).map(|&i| &self.enums[i])
    }

    pub fn get_type_by_name(&self, name: &str) -> Option<&TypeDefinition> {
        self.type_map.get(name).map(|&i| &self.types[i])
    }

    pub fn is_enum(&self, name: &str) -> bool {
        self.enum_map.contains_key(name)
    }

    pub fn is_type(&self, name: &str) -> bool {
        self.type_map.contains_key(name)
    }

    pub fn packet_count(&self) -> usize {
        self.packets.len()
    }

    pub fn enum_count(&self) -> usize {
        self.enums.len()
    }

    pub fn type_count(&self) -> usize {
        self.types.len()
    }
}

#[derive(Debug)]
pub enum SchemaError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidDirectory { path: String, reason: String },
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Json(e) => write!(f, "JSON error: {}", e),
            Self::InvalidDirectory { path, reason } => {
                write!(f, "invalid directory '{}': {}", path, reason)
            }
        }
    }
}

impl std::error::Error for SchemaError {}

impl From<std::io::Error> for SchemaError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for SchemaError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::FieldType;

    fn test_schema() -> Schema {
        let docs_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("docs").join("protocol-docs"))
            .expect("Cannot resolve protocol-docs path");
        Schema::load_all(&docs_path).expect("Failed to load protocol-docs")
    }

    #[test]
    fn test_load_all_counts() {
        let schema = test_schema();
        assert!(schema.packet_count() >= 150, "packets >= 150, got {}", schema.packet_count());
        assert!(schema.enum_count() >= 100, "enums >= 100, got {}", schema.enum_count());
        assert!(schema.type_count() >= 190, "types >= 190, got {}", schema.type_count());
    }

    #[test]
    fn test_get_login_packet() {
        let schema = test_schema();
        let pkt = schema.get_packet_by_id(1).expect("Login packet (id=1)");
        assert_eq!(pkt.name, "LoginPacket");
        assert_eq!(pkt.fields.len(), 2);
    }

    #[test]
    fn test_get_disconnect_packet() {
        let schema = test_schema();
        let pkt = schema.get_packet_by_id(5).expect("Disconnect packet (id=5)");
        assert_eq!(pkt.name, "DisconnectPacket");
        let messages_field = &pkt.fields[1];
        assert_eq!(messages_field.name, "Messages");
        match &messages_field.field_type {
            FieldType::SwitchCase(sc) => assert_eq!(sc.cases.len(), 2),
            _ => panic!("Expected SwitchCase"),
        }
    }

    #[test]
    fn test_get_enum() {
        let schema = test_schema();
        let r#enum = schema.get_enum_by_name("MinecraftPacketIds").expect("MinecraftPacketIds");
        assert!(r#enum.values.len() > 220);
        assert_eq!(r#enum.values[0].name, "KeepAlive");
        assert_eq!(r#enum.values[0].value, 0);
    }

    #[test]
    fn test_embedded_roundtrip() {
        let schema = test_schema();
        // Simulate what build.rs does: serialize, deserialize, verify counts
        let pcount = schema.packet_count();
        let ecount = schema.enum_count();
        let tcount = schema.type_count();
        assert!(pcount >= 150);
        assert!(ecount >= 100);
        assert!(tcount >= 190);
    }
}
