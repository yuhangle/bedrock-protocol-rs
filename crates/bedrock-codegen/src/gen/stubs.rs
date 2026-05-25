use bedrock_protocol_schema::{
    field::{FieldType, SwitchCaseBranch},
    Schema,
};

use std::collections::BTreeSet;
use crate::naming;

/// Generate stub type definitions for types referenced by packets but not defined.
pub fn generate_stubs(schema: &Schema) -> String {
    let mut referenced = BTreeSet::new();

    // Collect all type references from packet fields
    for packet in &schema.packets {
        for field in &packet.fields {
            collect_refs(&field.field_type, &mut referenced, schema);
        }
    }

    // Also collect from type definitions
    for type_def in &schema.types {
        for field in &type_def.fields {
            collect_refs(&field.field_type, &mut referenced, schema);
        }
    }

    // Remove types that are already handled
    let known: BTreeSet<String> = [
        "bool", "uint8", "int16", "uint16", "int32", "uint32", "int64", "uint64",
        "float", "double", "string", "varint32", "uvarint32", "varint64", "uvarint64",
        "int32_be", "uint32_be", "CompoundTag", "void",
        "ActorUniqueID", "ActorRuntimeID", "EntityNetId", "PositionTrackingId", "PlayerInputTick",
        "Vec3", "BlockPos", "mce::UUID",
    ].iter().map(|s| s.to_string()).collect();

    let mut output = String::new();
    output.push_str("// Auto-generated stubs for protocol types.\n");
    output.push_str("// Replace these with proper implementations as needed.\n");
    output.push_str("#![allow(non_camel_case_types, unused_imports, unused_must_use, unused_variables)]\n\n");
    output.push_str("use bedrock_common::{BedrockRead, BedrockWrite, BedrockSerializable, BinaryStreamError};\n\n");

    for type_name in &referenced {
        if known.contains(type_name.as_str()) { continue; }
        if schema.is_type(type_name) { continue; }
        if schema.is_enum(type_name) { continue; }
        if type_name.contains('<') || type_name.contains(',') { continue; }
        if type_name.starts_with(|c: char| c.is_lowercase()) { continue; }

        let struct_name = naming::to_rust_struct_name(type_name);
        // Skip if it's already defined in crate::types
        let predefined = ["Mce_UUID", "FullContainerName", "ConnectionRequest",
            "ItemStackRequest", "ItemStackRequestSlotInfo", "ItemStackRequestActionTransferBase",
            "ItemStackRequestAction", "ItemStackRequestData", "ItemData", "DisconnectPacketMessages"];
        if predefined.contains(&struct_name.as_str()) { continue; }

        output.push_str(&format!(
            "#[derive(Debug, Clone, PartialEq)]\n\
             pub struct {}(pub Vec<u8>);\n\n\
             impl BedrockSerializable for {} {{\n\
             fn write_to(&self, stream: &mut dyn BedrockWrite) -> Result<(), BinaryStreamError> {{\n\
                 stream.write_raw_bytes(&self.0)\n\
             }}\n\
             fn read_from(stream: &mut dyn BedrockRead) -> Result<Self, BinaryStreamError> {{\n\
                 Ok(Self(stream.read_remaining()?))\n\
             }}\n\
             }}\n\n",
            struct_name, struct_name));
    }

    output
}

fn collect_refs(field_type: &FieldType, refs: &mut BTreeSet<String>, schema: &Schema) {
    match field_type {
        FieldType::Named(name) => {
            // Only collect if it's a complex type (not a primitive)
            if !is_primitive_type(name) && !schema.is_enum(name) {
                refs.insert(name.clone());
            }
        }
        FieldType::SwitchCase(sc) => {
            for case in &sc.cases {
                match case {
                    SwitchCaseBranch::Type(s) => {
                        if !is_primitive_type(s) && !schema.is_enum(s) {
                            refs.insert(s.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
        FieldType::Map { key, value } => {
            if !is_primitive_type(key) { refs.insert(key.clone()); }
            if !is_primitive_type(value) { refs.insert(value.clone()); }
        }
    }
}

fn is_primitive_type(name: &str) -> bool {
    matches!(name,
        "bool" | "int8" | "uint8" | "int16" | "uint16" | "int32" | "uint32"
        | "int64" | "uint64" | "int" | "float" | "double" | "string"
        | "varint32" | "uvarint32" | "varint64" | "uvarint64"
        | "int32_be" | "uint32_be" | "CompoundTag"
        | "ActorUniqueID" | "ActorRuntimeID" | "EntityNetId"
        | "PositionTrackingId" | "PlayerInputTick"
    )
}
