use crate::field::FieldDefinition;
use serde::Deserialize;

/// A reusable type definition from `protocol-docs/types/*.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct TypeDefinition {
    /// The type name (e.g., `"Vec3"`, `"BlockPos"`).
    pub name: String,

    /// A list of fields in this type.
    #[serde(default)]
    pub fields: Vec<FieldDefinition>,
}
