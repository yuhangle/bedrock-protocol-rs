use crate::field::FieldDefinition;
use serde::Deserialize;

/// A packet definition from `protocol-docs/packets/*.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct PacketDefinition {
    /// The numeric packet ID.
    pub id: u32,

    /// The packet name (e.g., `"LoginPacket"`).
    pub name: String,

    /// A list of fields in the packet body (excluding the packet ID header).
    #[serde(default)]
    pub fields: Vec<FieldDefinition>,

    /// Optional notes about the packet.
    pub notes: Option<String>,
}
