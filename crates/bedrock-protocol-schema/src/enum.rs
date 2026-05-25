use serde::Deserialize;

/// An enum definition from `protocol-docs/enums/*.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct EnumDefinition {
    /// The enum name (e.g., `"Connection::DisconnectFailReason"`).
    pub name: String,

    /// The list of enum values.
    pub values: Vec<EnumValue>,
}

/// A single enum value.
#[derive(Debug, Clone, Deserialize)]
pub struct EnumValue {
    /// The name of this enum variant.
    pub name: String,

    /// The numeric value.
    pub value: i64,
}
