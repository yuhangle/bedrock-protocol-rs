use bedrock_protocol_schema::Schema;

/// Information about how to map a JSON type to Rust code.
pub struct TypeMapping {
    /// The Rust type (e.g., "i32", "String", "Vec3").
    pub rust_type: String,
    /// The read method call (e.g., "read_varint()").
    pub read_call: String,
    /// The write method call (e.g., "write_varint(value)").
    pub write_call: &'static str,
    /// Whether this is a "flat" primitive (can be read with a single method call).
    pub is_primitive: bool,
}

/// Get the type mapping for a named type (primitive or known type reference).
pub fn resolve_type(name: &str, schema: &Schema) -> TypeMapping {
    // Check known primitives first
    if let Some(mapping) = primitive_mapping(name) {
        return mapping;
    }

    // Check if it's an enum (C++ namespace style)
    if schema.is_enum(name) {
        let enum_name = super::naming::to_rust_enum_name(name);
        return TypeMapping {
            rust_type: enum_name,
            read_call: format!("/* enum {} */ read_varint()", name),
            write_call: "/* enum */ write_varint",
            is_primitive: false,
        };
    }

    // Default: it's a type reference
    TypeMapping {
        rust_type: super::naming::to_rust_struct_name(name),
        read_call: format!("/* type */ {}", name),
        write_call: "/* type */ write",
        is_primitive: false,
    }
}

/// Get the read method for a primitive type.
pub fn read_method(primitive: &str) -> &'static str {
    match primitive {
        "bool" => "read_bool",
        "uint8" | "byte" | "unsigned_char" => "read_u8",
        "int16" | "signed_short" => "read_i16",
        "uint16" | "unsigned_short" | "short" => "read_u16",
        "int32" | "signed_int" | "int" => "read_i32",
        "uint32" | "unsigned_int" => "read_u32",
        "int64" | "signed_int64" => "read_i64",
        "uint64" | "unsigned_int64" => "read_u64",
        "int32_be" | "signed_big_endian_int" => "read_i32_be",
        "uint32_be" => "read_u32_be",
        "float" => "read_f32",
        "double" => "read_f64",
        "varint32" | "varint" => "read_varint",
        "uvarint32" | "unsigned_varint" => "read_unsigned_varint",
        "varint64" => "read_varint64",
        "uvarint64" | "unsigned_varint64" => "read_unsigned_varint64",
        "normalized_float" => "read_normalized_f32",
        "string" => "read_string",
        "short_string" => "read_short_string",
        "long_string" => "read_long_string",
        "u24" | "unsigned_int24" => "read_u24",
        _ => "read_u8", // fallback
    }
}

/// Get the write method for a primitive type.
pub fn write_method(primitive: &str) -> &'static str {
    match primitive {
        "bool" => "write_bool",
        "uint8" | "byte" | "unsigned_char" => "write_u8",
        "int16" | "signed_short" => "write_i16",
        "uint16" | "unsigned_short" | "short" => "write_u16",
        "int32" | "signed_int" | "int" => "write_i32",
        "uint32" | "unsigned_int" => "write_u32",
        "int64" | "signed_int64" => "write_i64",
        "uint64" | "unsigned_int64" => "write_u64",
        "int32_be" | "signed_big_endian_int" => "write_i32_be",
        "uint32_be" => "write_u32_be",
        "float" => "write_f32",
        "double" => "write_f64",
        "varint32" | "varint" => "write_varint",
        "uvarint32" | "unsigned_varint" => "write_unsigned_varint",
        "varint64" => "write_varint64",
        "uvarint64" | "unsigned_varint64" => "write_unsigned_varint64",
        "normalized_float" => "write_normalized_f32",
        "string" => "write_string",
        "short_string" => "write_short_string",
        "long_string" => "write_long_string",
        "u24" | "unsigned_int24" => "write_u24",
        _ => "write_raw_bytes", // fallback
    }
}

/// Get the Rust type for a primitive JSON type name.
pub fn rust_primitive_type(primitive: &str) -> &'static str {
    match primitive {
        "bool" => "bool",
        "uint8" | "byte" | "unsigned_char" => "u8",
        "int16" | "signed_short" => "i16",
        "uint16" | "unsigned_short" | "short" => "u16",
        "int32" | "signed_int" | "int" | "int32_be" | "signed_big_endian_int" | "varint32"
        | "varint" => "i32",
        "uint32" | "unsigned_int" | "uint32_be" | "uvarint32" | "unsigned_varint" => "u32",
        "u24" | "unsigned_int24" => "u32",
        "int64" | "signed_int64" | "varint64" => "i64",
        "uint64" | "unsigned_int64" | "uvarint64" | "unsigned_varint64" => "u64",
        "float" | "normalized_float" => "f32",
        "double" => "f64",
        "string" | "short_string" | "long_string" => "String",
        _ => "Vec<u8>", // fallback for unknown types
    }
}

/// Check if a type name is a known primitive and return its mapping.
pub fn primitive_mapping(name: &str) -> Option<TypeMapping> {
    let rust_type = rust_primitive_type(name);
    let read = read_method(name);
    let write = write_method(name);

    // If the fallback was used (write_raw_bytes), this isn't a recognized primitive
    if write == "write_raw_bytes" && name != "raw_bytes" {
        return None;
    }

    Some(TypeMapping {
        rust_type: rust_type.to_string(),
        read_call: format!("stream.{}()", read),
        write_call: write,
        is_primitive: true,
    })
}

/// Special type names that should be treated as integer aliases rather than struct types.
pub fn is_integer_alias(name: &str) -> bool {
    matches!(
        name,
        "ActorUniqueID"
            | "ActorRuntimeID"
            | "EntityNetId"
            | "PositionTrackingId"
            | "PlayerInputTick"
    )
}

/// Get the underlying integer type for an alias.
pub fn integer_alias_type(name: &str) -> &'static str {
    match name {
        "ActorUniqueID" | "ActorRuntimeID" => "i64",
        "EntityNetId" | "PositionTrackingId" | "PlayerInputTick" => "i32",
        _ => "i64",
    }
}

/// Known type name mappings from protocol JSON names to Rust type names.
/// These override the automatic name conversion.
pub fn known_type_mapping(name: &str) -> Option<&'static str> {
    match name {
        "mce::UUID" => Some("Uuid"),
        "mce::Color" => Some("Uuid"), // placeholder
        _ => None,
    }
}

/// Resolve a type name to its Rust type path for use in generated code.
pub fn resolve_rust_type_name(name: &str) -> String {
    if let Some(mapped) = known_type_mapping(name) {
        return mapped.to_string();
    }
    if is_integer_alias(name) {
        return integer_alias_type(name).to_string();
    }
    let prim = rust_primitive_type(name);
    if prim != "Vec<u8>" || name == "raw_bytes" {
        return prim.to_string();
    }
    super::naming::to_rust_struct_name(name)
}

pub fn to_rust_struct_name(name: &str) -> String {
    super::naming::to_rust_struct_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_mapping_bool() {
        let m = primitive_mapping("bool").unwrap();
        assert_eq!(m.rust_type, "bool");
        assert_eq!(m.read_call, "stream.read_bool()");
    }

    #[test]
    fn test_primitive_mapping_varint() {
        let m = primitive_mapping("varint32").unwrap();
        assert_eq!(m.rust_type, "i32");
        assert_eq!(m.read_call, "stream.read_varint()");
    }

    #[test]
    fn test_primitive_mapping_uvarint() {
        let m = primitive_mapping("uvarint32").unwrap();
        assert_eq!(m.rust_type, "u32");
        assert_eq!(m.read_call, "stream.read_unsigned_varint()");
    }

    #[test]
    fn test_primitive_mapping_string() {
        let m = primitive_mapping("string").unwrap();
        assert_eq!(m.rust_type, "String");
        assert_eq!(m.read_call, "stream.read_string()");
    }

    #[test]
    fn test_primitive_mapping_float() {
        let m = primitive_mapping("float").unwrap();
        assert_eq!(m.rust_type, "f32");
        assert_eq!(m.read_call, "stream.read_f32()");
    }

    #[test]
    fn test_integer_aliases() {
        assert!(is_integer_alias("ActorUniqueID"));
        assert!(is_integer_alias("ActorRuntimeID"));
        assert!(!is_integer_alias("Vec3"));
    }
}
