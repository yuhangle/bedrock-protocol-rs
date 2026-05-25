use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::Deserialize;
use std::fmt;

/// A single field within a packet or type definition.
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    /// The field name (e.g., `"Client Network Version"`).
    pub name: String,

    /// The resolved type of this field.
    pub field_type: FieldType,

    /// If this field references an enum, the enum's name.
    pub enum_ref: Option<String>,

    /// If true, this field may not be present.
    pub optional: bool,

    /// If set, this field is a list with the given length-prefix type.
    pub repeat: Option<RepeatInfo>,

    /// Repeat information embedded inside the type object itself.
    /// e.g., `"type": {"type": "X", "repeat": {"prefix": "uvarint32"}}`
    pub type_repeat: Option<RepeatInfo>,

    /// Optional additional constraints.
    pub constraints: Option<serde_json::Value>,
}

/// Length-prefix or fixed-count information for a repeated field.
#[derive(Debug, Clone)]
pub struct RepeatInfo {
    /// The type used for the length prefix (e.g., `"uvarint32"`).
    /// If None, the repeat has a fixed count.
    pub prefix: Option<String>,

    /// A fixed number of elements (instead of a prefix).
    /// e.g., `"repeat": {"count": 9}` means exactly 9 elements.
    pub count: Option<u32>,
}

impl<'de> Deserialize<'de> for RepeatInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Use serde_json::Value as intermediary to handle "prefix" | "count"
        let v: serde_json::Value = Deserialize::deserialize(deserializer)?;
        let prefix = v.get("prefix").and_then(|p| p.as_str()).map(|s| s.to_string());
        let count = v.get("count").and_then(|c| c.as_u64()).map(|n| n as u32);

        if prefix.is_none() && count.is_none() {
            return Err(de::Error::custom(
                "RepeatInfo must have 'prefix' or 'count' field",
            ));
        }

        Ok(RepeatInfo { prefix, count })
    }
}

/// The type of a protocol field.
#[derive(Debug, Clone)]
pub enum FieldType {
    /// A primitive or named type (e.g., `"bool"`, `"uvarint32"`, `"Vec3"`, `"BlockPos"`).
    Named(String),

    /// A switch/case conditional type.
    SwitchCase(SwitchCase),

    /// A map/dictionary type with key/value types (e.g., `{"key": "uint16", "value": "Type"}`).
    Map {
        /// The key type (usually a primitive like `"uint16"` or `"string"`).
        key: String,
        /// The value type (a primitive or type name).
        value: String,
    },
}

/// A switch/case conditional type.
///
/// The discriminator value determines which case to use. Cases can be:
/// - `None` (null): no data for this branch
/// - `Some(name)`: a named type or primitive
#[derive(Debug, Clone)]
pub struct SwitchCase {
    /// The type of the discriminator field (usually `uvarint32`).
    pub switch_type: Box<FieldType>,

    /// Optional enum reference for the discriminator values.
    pub switch_enum: Option<String>,

    /// Optional name for the discriminator field.
    pub switch_name: Option<String>,

    /// The cases, indexed by discriminator value.
    pub cases: Vec<SwitchCaseBranch>,
}

/// A single branch in a switch/case type.
#[derive(Debug, Clone)]
pub enum SwitchCaseBranch {
    /// No data for this branch.
    Empty,
    /// A named primitive type (e.g., `"bool"`, `"int"`, `"float"`).
    Primitive(String),
    /// A reference to a named type.
    Type(String),
}

// ---------------------------------------------------------------------------
// Custom Deserialize for FieldDefinition (handles complex type field)
// ---------------------------------------------------------------------------

impl<'de> Deserialize<'de> for FieldDefinition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Name,
            #[serde(rename = "type")]
            Type,
            #[serde(rename = "enum")]
            Enum,
            Optional,
            Repeat,
            Constraints,
            #[serde(other)]
            Ignored,
        }

        struct FieldDefinitionVisitor;

        impl<'de> Visitor<'de> for FieldDefinitionVisitor {
            type Value = FieldDefinition;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a field definition")
            }

            fn visit_map<V>(self, mut map: V) -> Result<FieldDefinition, V::Error>
            where
                V: MapAccess<'de>,
            {
                use serde::de::Error;

                let mut name: Option<String> = None;
                let mut field_type: Option<FieldType> = None;
                let mut type_repeat: Option<RepeatInfo> = None;
                let mut enum_ref: Option<String> = None;
                let mut optional: Option<bool> = None;
                let mut repeat: Option<RepeatInfo> = None;
                let mut constraints: Option<serde_json::Value> = None;

                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::Name => {
                            name = Some(map.next_value()?);
                        }
                        Field::Type => {
                            // Read type as raw JSON to detect inner-repeat patterns
                            let raw: serde_json::Value = map.next_value()?;
                            match &raw {
                                serde_json::Value::Object(obj) => {
                                    // Object type — could be switch/case, map, or type+repeat
                                    if let Some(inner_type) = obj.get("type").and_then(|v| v.as_str()) {
                                        // {"type": "X", "repeat": {...}} — extract inner type + repeat
                                        if let Some(rep) = obj.get("repeat") {
                                            type_repeat = Some(serde_json::from_value(rep.clone())
                                                .map_err(|e| Error::custom(format!("invalid type_repeat: {}", e)))?);
                                        }
                                        field_type = Some(FieldType::Named(inner_type.to_string()));
                                    } else {
                                        // Standard switch/case or map: deserialize as FieldType
                                        field_type = Some(serde_json::from_value(raw).map_err(|e| {
                                            Error::custom(format!("invalid field type object: {}", e))
                                        })?);
                                    }
                                }
                                _ => {
                                    // String type: deserialize normally
                                    field_type = Some(serde_json::from_value(raw).map_err(|e| {
                                        Error::custom(format!("invalid field type: {}", e))
                                    })?);
                                }
                            }
                        }
                        Field::Enum => {
                            enum_ref = Some(map.next_value()?);
                        }
                        Field::Optional => {
                            optional = Some(map.next_value()?);
                        }
                        Field::Repeat => {
                            repeat = Some(map.next_value()?);
                        }
                        Field::Constraints => {
                            constraints = Some(map.next_value()?);
                        }
                        Field::Ignored => {
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                let name = name.ok_or_else(|| Error::missing_field("name"))?;
                let field_type = field_type.unwrap_or(FieldType::Named("void".to_string()));

                Ok(FieldDefinition {
                    name,
                    field_type,
                    enum_ref,
                    optional: optional.unwrap_or(false),
                    repeat,
                    type_repeat,
                    constraints,
                })
            }
        }

        deserializer.deserialize_map(FieldDefinitionVisitor)
    }
}

// ---------------------------------------------------------------------------
// Custom Deserialize for FieldType (string or switch/case or map object)
// ---------------------------------------------------------------------------

impl<'de> Deserialize<'de> for FieldType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FieldTypeVisitor;

        impl<'de> Visitor<'de> for FieldTypeVisitor {
            type Value = FieldType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a type string or type object (switch/case or map)")
            }

            fn visit_str<E>(self, value: &str) -> Result<FieldType, E>
            where
                E: de::Error,
            {
                Ok(FieldType::Named(value.to_string()))
            }

            fn visit_map<V>(self, mut map: V) -> Result<FieldType, V::Error>
            where
                V: MapAccess<'de>,
            {
                use serde::de::Error;

                // Collect all key-value pairs into a flat serde_json::Map for inspection
                let mut raw_map = serde_json::Map::new();
                while let Some((key, value)) = map.next_entry::<String, serde_json::Value>()? {
                    raw_map.insert(key, value);
                }

                // Determine type based on keys present
                if raw_map.contains_key("switch") {
                    // Switch/case type
                    let switch_val = raw_map.get("switch").unwrap();
                    let cases_val = raw_map.get("cases").ok_or_else(|| {
                        Error::custom("switch/case type missing 'cases' field")
                    })?;

                    let switch_type_str = switch_val
                        .get("type")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| Error::custom("switch field missing 'type'"))?
                        .to_string();
                    let switch_type = FieldType::Named(switch_type_str);

                    let switch_enum = switch_val
                        .get("enum")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let switch_name = switch_val
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let case_list: Vec<Option<String>> =
                        serde_json::from_value(cases_val.clone())
                            .map_err(|e| Error::custom(format!("invalid cases: {}", e)))?;

                    let branches: Vec<SwitchCaseBranch> = case_list
                        .into_iter()
                        .map(|c| match c {
                            None => SwitchCaseBranch::Empty,
                            Some(s) => match s.as_str() {
                                "bool" | "int" | "int8" | "uint8" | "int16" | "uint16"
                                | "int32" | "uint32" | "int64" | "uint64"
                                | "float" | "double" | "string" | "varint32"
                                | "uvarint32" | "varint64" | "uvarint64"
                                | "int32_be" | "uint32_be" => SwitchCaseBranch::Primitive(s),
                                _ => SwitchCaseBranch::Type(s),
                            },
                        })
                        .collect();

                    Ok(FieldType::SwitchCase(SwitchCase {
                        switch_type: Box::new(switch_type),
                        switch_enum,
                        switch_name,
                        cases: branches,
                    }))
                } else if raw_map.contains_key("key") && raw_map.contains_key("value") {
                    // Map type
                    let key_str = raw_map["key"]
                        .as_str()
                        .ok_or_else(|| Error::custom("map 'key' must be a string"))?
                        .to_string();
                    let value_str = raw_map["value"]
                        .as_str()
                        .ok_or_else(|| Error::custom("map 'value' must be a string"))?
                        .to_string();
                    Ok(FieldType::Map {
                        key: key_str,
                        value: value_str,
                    })
                } else {
                    Err(Error::custom(format!(
                        "unknown type object with keys: {:?}",
                        raw_map.keys().collect::<Vec<_>>()
                    )))
                }
            }
        }

        deserializer.deserialize_any(FieldTypeVisitor)
    }
}
