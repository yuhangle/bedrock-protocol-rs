/// Convert a field name like "Client Network Version" to snake_case "client_network_version".
pub fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    let mut prev_upper = false;
    let mut prev_char = false;

    for (i, ch) in name.char_indices() {
        if ch.is_uppercase() {
            if prev_char && !prev_upper && i > 0 {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
            prev_upper = true;
            prev_char = true;
        } else if ch.is_lowercase() || ch.is_numeric() {
            result.push(ch);
            prev_upper = false;
            prev_char = true;
        } else if ch == '[' || ch == ']' || ch == '(' || ch == ')' || ch == ':' || ch == ';' || ch == '"' || ch == ',' {
            // Skip delimiters — act as word boundaries without emitting a character
            prev_upper = false;
            prev_char = false;
        } else if ch == ' ' || ch == '-' || ch == '.' || ch == '/' || ch == '\'' || ch == '?' {
            if prev_char {
                result.push('_');
            }
            prev_upper = false;
            prev_char = false;
        } else {
            result.push(ch);
            prev_upper = false;
            prev_char = true;
        }
    }

    // Clean up: collapse multiple underscores, trim trailing
    let cleaned: String = result
        .chars()
        .fold(String::new(), |mut acc, c| {
            if c == '_' && acc.ends_with('_') {
                // skip duplicate
            } else {
                acc.push(c);
            }
            acc
        })
        .trim_end_matches('_')
        .to_string();

    // Ensure it's not a Rust keyword
    match cleaned.as_str() {
        "type" => "r#type".to_string(),
        "enum" => "r#enum".to_string(),
        "match" => "r#match".to_string(),
        "ref" => "r#ref".to_string(),
        "let" => "r#let".to_string(),
        "mut" => "r#mut".to_string(),
        "return" => "r#return".to_string(),
        "static" => "r#static".to_string(),
        "loop" => "r#loop".to_string(),
        "while" => "r#while".to_string(),
        "for" => "r#for".to_string(),
        "in" => "in_".to_string(),
        "if" => "r#if".to_string(),
        "else" => "r#else".to_string(),
        "impl" => "r#impl".to_string(),
        "trait" => "r#trait".to_string(),
        "struct" => "r#struct".to_string(),
        "self" => "self_".to_string(),
        "super" => "super_".to_string(),
        "crate" => "crate_".to_string(),
        _ => cleaned,
    }
}

/// Convert a C++ namespace name like "Connection::DisconnectFailReason"
/// to a valid Rust struct/enum name "ConnectionDisconnectFailReason".
/// Ensures PascalCase: first character is always uppercase.
pub fn to_rust_enum_name(name: &str) -> String {
    let cleaned = name
        .replace("::", "")
        .replace("__", "_")
        .replace('.', "_")
        .replace('<', "_")
        .replace('>', "_")
        .replace(',', "_")
        .replace(' ', "_")
        .replace('-', "_")
        .trim_end_matches('_')
        .to_string();
    // PascalCase: uppercase first char
    let mut result = String::new();
    for (i, ch) in cleaned.char_indices() {
        if i == 0 && ch.is_lowercase() {
            result.push(ch.to_ascii_uppercase());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Known type name overrides for commonly referenced types.
pub fn known_rust_type_name(name: &str) -> Option<&'static str> {
    match name {
        "mce::UUID" | "mce__UUID" => Some("Uuid"),
        _ => None,
    }
}

/// Convert a struct/type name to a valid Rust struct name.
/// Most names are already PascalCase.
pub fn to_rust_struct_name(name: &str) -> String {
    // Check known overrides first
    if let Some(mapped) = known_rust_type_name(name) {
        return mapped.to_string();
    }
    // Remove C++ namespace prefixes
    let cleaned = name
        .replace("::", "_")
        .replace('.', "_")
        .replace('<', "_")
        .replace('>', "_");

    // Ensure it starts with uppercase
    let mut result = String::new();
    for (i, ch) in cleaned.char_indices() {
        if i == 0 && ch.is_lowercase() {
            result.push(ch.to_ascii_uppercase());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Sanitize an enum variant name: handle special prefixes like "INTERNAL_", "TESTONLY_",
/// and convert word separators (spaces, hyphens, etc.) to PascalCase.
pub fn sanitize_enum_variant(name: &str) -> String {
    let cleaned = name
        .trim_start_matches("INTERNAL_")
        .trim_start_matches("TESTONLY_");

    if cleaned.is_empty() {
        return name.to_string();
    }

    let mut result = String::new();
    let mut upper_next = true;
    for ch in cleaned.chars() {
        if ch == '_' || ch == ' ' || ch == '-' || ch == '/' || ch == '.' {
            upper_next = true;
        } else if upper_next {
            result.push(ch.to_ascii_uppercase());
            upper_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case_simple() {
        assert_eq!(to_snake_case("Client Network Version"), "client_network_version");
    }

    #[test]
    fn test_snake_case_camel() {
        assert_eq!(to_snake_case("TargetActorID"), "target_actor_id");
    }

    #[test]
    fn test_snake_case_with_abbrevs() {
        assert_eq!(to_snake_case("ActorUniqueID"), "actor_unique_id");
    }

    #[test]
    fn test_snake_case_keyword_type() {
        assert_eq!(to_snake_case("Type"), "r#type");
    }

    #[test]
    fn test_snake_case_parentheses() {
        assert_eq!(to_snake_case("Behavior Tree Structure (JSON)"), "behavior_tree_structure_json");
        assert_eq!(to_snake_case("UpdateBlock (Legacy)"), "update_block_legacy");
    }

    #[test]
    fn test_rust_enum_name() {
        assert_eq!(
            to_rust_enum_name("Connection::DisconnectFailReason"),
            "ConnectionDisconnectFailReason"
        );
    }

    #[test]
    fn test_sanitize_enum_variant() {
        assert_eq!(sanitize_enum_variant("INTERNAL_NoFailOccurred"), "NoFailOccurred");
        assert_eq!(sanitize_enum_variant("Login"), "Login");
    }
}
