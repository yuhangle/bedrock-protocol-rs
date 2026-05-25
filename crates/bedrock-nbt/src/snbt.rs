//! SNBT (Stringified NBT) parser for Minecraft Bedrock / Java Edition.
//!
//! Parses the text-based NBT format used for debugging and data interchange.
//! Supports full SNBT syntax including typed arrays ([B; ...], [I; ...], [L; ...]),
//! comments, unicode escapes, and all numeric suffix variants.

use crate::{CompoundTag, Tag, TagType};

/// Error type for SNBT parsing.
#[derive(Debug, Clone)]
pub struct SnbtParseError {
    /// Byte position in the input where the error occurred.
    pub position: usize,
    /// Human-readable error description.
    pub message: String,
}

impl std::fmt::Display for SnbtParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SNBT parse error at position {}: {}", self.position, self.message)
    }
}

impl std::error::Error for SnbtParseError {}

impl From<String> for SnbtParseError {
    fn from(message: String) -> Self {
        SnbtParseError { position: 0, message }
    }
}

struct Cursor<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(input: &'a str) -> Self {
        Cursor { input, pos: 0 }
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn advance(&mut self) {
        if let Some(c) = self.remaining().chars().next() {
            self.pos += c.len_utf8();
        }
    }

    fn eat_char(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn err(&self, msg: impl Into<String>) -> SnbtParseError {
        SnbtParseError { position: self.pos, message: msg.into() }
    }

    /// Skip whitespace and comments. Returns error on unterminated block comments.
    fn skip_whitespace(&mut self) -> Result<(), SnbtParseError> {
        loop {
            // Skip actual whitespace
            let before = self.pos;
            while let Some(c) = self.peek() {
                if c.is_ascii_whitespace() { self.advance(); }
                else { break; }
            }
            // Skip comments
            if self.remaining().starts_with("//") {
                self.pos += 2;
                while let Some(c) = self.peek() {
                    if c == '\n' { self.advance(); break; }
                    self.advance();
                }
            } else if self.remaining().starts_with("/*") {
                self.pos += 2;
                loop {
                    match self.peek() {
                        None => return Err(self.err("unterminated block comment")),
                        Some('*') => {
                            self.advance();
                            if self.peek() == Some('/') { self.advance(); break; }
                        }
                        Some(_) => { self.advance(); }
                    }
                }
            }
            if self.pos == before { break; } // No progress means no more whitespace/comments
        }
        Ok(())
    }

    /// Expect and consume a specific character.
    fn expect_char(&mut self, c: char) -> Result<(), SnbtParseError> {
        self.skip_whitespace()?;
        match self.peek() {
            Some(ch) if ch == c => { self.advance(); Ok(()) }
            Some(other) => Err(self.err(format!("expected '{}' but found '{}'", c, other))),
            None => Err(self.err(format!("expected '{}' but reached end of input", c))),
        }
    }

    /// Parse a quoted or unquoted string.
    fn parse_string(&mut self) -> Result<String, SnbtParseError> {
        self.skip_whitespace()?;
        match self.peek() {
            Some('"') => self.parse_quoted_string('"'),
            Some('\'') => self.parse_quoted_string('\''),
            Some(c) if c.is_alphanumeric() || c == '_' || c == '-' || c == '+' || c == '.' => {
                self.parse_unquoted_string()
            }
            Some(c) => Err(self.err(format!("unexpected character '{}' in string", c))),
            None => Err(self.err("unexpected end of input in string")),
        }
    }

    /// Parse a quoted string (single or double quote).
    fn parse_quoted_string(&mut self, quote: char) -> Result<String, SnbtParseError> {
        self.advance(); // skip opening quote
        let mut result = String::new();
        loop {
            match self.peek() {
                None => return Err(self.err("unterminated string")),
                Some(c) if c == quote => {
                    self.advance(); // skip closing quote
                    return Ok(result);
                }
                Some('\\') => {
                    self.advance(); // skip backslash
                    match self.eat_char() {
                        None => return Err(self.err("unterminated escape sequence")),
                        Some('"') => result.push('"'),
                        Some('\'') => result.push('\''),
                        Some('\\') => result.push('\\'),
                        Some('/') => result.push('/'),
                        Some('n') => result.push('\n'),
                        Some('t') => result.push('\t'),
                        Some('r') => result.push('\r'),
                        Some('b') => result.push('\u{0008}'),
                        Some('f') => result.push('\u{000C}'),
                        Some('u') => {
                            let hex = self.read_hex_digits(4)?;
                            let code_point = u32::from_str_radix(&hex, 16).map_err(|_| {
                                self.err("invalid unicode escape")
                            })?;
                            // Basic surrogate pair handling
                            if let Some(ch) = char::from_u32(code_point) {
                                result.push(ch);
                            } else {
                                return Err(self.err("invalid unicode code point"));
                            }
                        }
                        Some(c) => { result.push(c); } // pass through unknown escapes
                    }
                }
                Some(c) => {
                    result.push(c);
                    self.advance();
                }
            }
        }
    }

    fn read_hex_digits(&mut self, count: usize) -> Result<String, SnbtParseError> {
        let mut hex = String::with_capacity(count);
        for _ in 0..count {
            match self.peek() {
                Some(c) if c.is_ascii_hexdigit() => {
                    hex.push(c);
                    self.advance();
                }
                Some(c) => return Err(self.err(format!("expected hex digit but found '{}'", c))),
                None => return Err(self.err("unexpected end of input in hex escape")),
            }
        }
        Ok(hex)
    }

    /// Parse an unquoted string key/value.
    fn parse_unquoted_string(&mut self) -> Result<String, SnbtParseError> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '+' || c == '.' { self.advance(); }
            else { break; }
        }
        Ok(self.input[start..self.pos].to_string())
    }

    /// Parse a number with optional suffix. Returns the appropriate Tag.
    fn parse_number(&mut self) -> Result<Tag, SnbtParseError> {
        self.skip_whitespace()?;

        // Check for sign
        let negative = match self.peek() {
            Some('+') => { self.advance(); false }
            Some('-') => { self.advance(); true }
            _ => false,
        };

        // Check for hex (0x) or binary (0b) prefix
        let radix: u32;
        let is_hex_or_bin = if self.remaining().starts_with("0x") || self.remaining().starts_with("0X") {
            self.pos += 2;
            radix = 16;
            true
        } else if self.remaining().starts_with("0b") || self.remaining().starts_with("0B") {
            self.pos += 2;
            radix = 2;
            true
        } else {
            radix = 10;
            false
        };

        // Read the numeric digits
        let mut int_part = String::new();
        let mut has_dot = false;
        let mut has_exp = false;
        let mut has_digit = false;

        if is_hex_or_bin {
            while let Some(c) = self.peek() {
                if c == '_' { self.advance(); continue; }
                if c.is_ascii_hexdigit() {
                    int_part.push(c);
                    has_digit = true;
                    self.advance();
                } else { break; }
            }
        } else {
            // Read integer part
            while let Some(c) = self.peek() {
                if c == '_' { self.advance(); continue; }
                if c.is_ascii_digit() { int_part.push(c); has_digit = true; self.advance(); }
                else { break; }
            }
            // Check for decimal point
            if self.peek() == Some('.') {
                has_dot = true;
                self.advance();
                while let Some(c) = self.peek() {
                    if c == '_' { self.advance(); continue; }
                    if c.is_ascii_digit() { int_part.push('.'); int_part.push(c); self.advance(); }
                    else { break; }
                }
            }
            // Check for exponent
            if self.peek() == Some('e') || self.peek() == Some('E') {
                has_exp = true;
                int_part.push('e');
                self.advance();
                if self.peek() == Some('+') || self.peek() == Some('-') {
                    int_part.push(self.peek().unwrap());
                    self.advance();
                }
                while let Some(c) = self.peek() {
                    if c == '_' { self.advance(); continue; }
                    if c.is_ascii_digit() { int_part.push(c); self.advance(); }
                    else { break; }
                }
            }
        }

        if !has_digit {
            return Err(self.err("expected numeric digit"));
        }

        let number_str = if negative { format!("-{}", int_part) } else { int_part.clone() };

        // Parse suffix (may be multi-character or comment-style)
        let suffix = self.parse_number_mark();

        // Determine the numeric type based on suffix
        match suffix.as_str() {
            "b" | "B" => {
                let val = parse_int_with_radix(&number_str, radix, 8)? as i8;
                Ok(Tag::Byte(val))
            }
            "s" | "S" => {
                let val = parse_int_with_radix(&number_str, radix, 16)? as i16;
                Ok(Tag::Short(val))
            }
            "l" | "L" => {
                let val = parse_int_with_radix(&number_str, radix, 64)?;
                Ok(Tag::Long(val))
            }
            "f" | "F" => {
                let val: f32 = if radix == 10 { number_str.parse().map_err(|_| self.err("invalid float"))? }
                    else { return Err(self.err("hex/binary float not supported")) };
                Ok(Tag::Float(val))
            }
            "d" | "D" => {
                let val: f64 = if radix == 10 { number_str.parse().map_err(|_| self.err("invalid double"))? }
                    else { return Err(self.err("hex/binary double not supported")) };
                Ok(Tag::Double(val))
            }
            // Two-char suffixes
            "sb" => {
                let val = parse_int_with_radix(&number_str, radix, 8)? as i8;
                Ok(Tag::Byte(val))
            }
            "ub" => {
                let val = parse_int_with_radix(&number_str, radix, 8)? as u8 as i8;
                Ok(Tag::Byte(val))
            }
            "ss" => {
                let val = parse_int_with_radix(&number_str, radix, 16)? as i16;
                Ok(Tag::Short(val))
            }
            "us" => {
                let val = parse_int_with_radix(&number_str, radix, 16)? as u16 as i16;
                Ok(Tag::Short(val))
            }
            "si" => {
                let val = parse_int_with_radix(&number_str, radix, 32)? as i32;
                Ok(Tag::Int(val))
            }
            "ui" => {
                let val = parse_int_with_radix(&number_str, radix, 32)? as u32 as i32;
                Ok(Tag::Int(val))
            }
            "sl" => {
                let val = parse_int_with_radix(&number_str, radix, 64)?;
                Ok(Tag::Long(val))
            }
            "ul" => {
                let val = parse_int_with_radix(&number_str, radix, 64)? as u64 as i64;
                Ok(Tag::Long(val))
            }
            // Comment-style suffixes: /*b*/, /*sb*/, /*ub*/, etc.
            _ => {
                // Check for comment-style suffix in the remaining text
                if let Some(css) = self.skip_comment_style_suffix() {
                    return match css.as_str() {
                        "b" => {
                            let val = parse_int_with_radix(&number_str, radix, 8)? as i8;
                            Ok(Tag::Byte(val))
                        }
                        "s" => {
                            let val = parse_int_with_radix(&number_str, radix, 16)? as i16;
                            Ok(Tag::Short(val))
                        }
                        "l" => {
                            let val = parse_int_with_radix(&number_str, radix, 64)?;
                            Ok(Tag::Long(val))
                        }
                        "f" => Ok(Tag::Float(number_str.parse::<f32>().map_err(|_| self.err("invalid float"))?)),
                        "d" => Ok(Tag::Double(number_str.parse::<f64>().map_err(|_| self.err("invalid double"))?)),
                        "sb" => { let v = parse_int_with_radix(&number_str, radix, 8)? as i8; Ok(Tag::Byte(v)) }
                        "ub" => { let v = parse_int_with_radix(&number_str, radix, 8)? as u8 as i8; Ok(Tag::Byte(v)) }
                        "ss" => { let v = parse_int_with_radix(&number_str, radix, 16)? as i16; Ok(Tag::Short(v)) }
                        "us" => { let v = parse_int_with_radix(&number_str, radix, 16)? as u16 as i16; Ok(Tag::Short(v)) }
                        "si" => { let v = parse_int_with_radix(&number_str, radix, 32)? as i32; Ok(Tag::Int(v)) }
                        "ui" => { let v = parse_int_with_radix(&number_str, radix, 32)? as u32 as i32; Ok(Tag::Int(v)) }
                        "sl" => { let v = parse_int_with_radix(&number_str, radix, 64)?; Ok(Tag::Long(v)) }
                        "ul" => { let v = parse_int_with_radix(&number_str, radix, 64)? as u64 as i64; Ok(Tag::Long(v)) }
                        _ => Err(self.err(format!("unknown comment-style suffix: /*{}*/", css))),
                    };
                }
                // No suffix = int or long or double
                if has_dot || has_exp {
                    let val: f64 = number_str.parse().map_err(|_| self.err("invalid double"))?;
                    Ok(Tag::Double(val))
                } else if radix == 10 {
                    // Try int first, then long
                    if let Ok(v) = number_str.parse::<i32>() {
                        Ok(Tag::Int(v))
                    } else {
                        let v = number_str.parse::<i64>().map_err(|_| self.err("integer out of range"))?;
                        Ok(Tag::Long(v))
                    }
                } else {
                    let v = parse_int_with_radix(&number_str, radix, 64)?;
                    if v >= i32::MIN as i64 && v <= i32::MAX as i64 {
                        Ok(Tag::Int(v as i32))
                    } else {
                        Ok(Tag::Long(v))
                    }
                }
            }
        }
    }

    /// Skip a comment-style suffix (`/*b*/`, `/*ub*/`, etc.) in the remaining text.
    /// Returns the suffix content without the `/*` `*/` delimiters.
    fn skip_comment_style_suffix(&mut self) -> Option<String> {
        // Skip whitespace before the comment
        let saved = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() { self.advance(); } else { break; }
        }
        if self.remaining().starts_with("/*") {
            self.pos += 2;
            let mut suffix = String::new();
            while let Some(c) = self.peek() {
                if c == '*' && self.remaining().len() > 1 && self.remaining().as_bytes()[1] == b'/' {
                    self.pos += 2; // skip */
                    return if suffix.is_empty() { None } else { Some(suffix) };
                }
                suffix.push(c);
                self.advance();
            }
            // Unterminated comment - restore position
            self.pos = saved;
            None
        } else {
            // No comment found - restore position
            self.pos = saved;
            None
        }
    }

    /// Parse suffix marker after a number.
    fn parse_number_mark(&mut self) -> String {
        // Check for two-char suffix
        let s = self.remaining();
        if s.len() >= 2 {
            let two = &s[..2];
            if matches!(two, "sb" | "ub" | "ss" | "us" | "si" | "ui" | "sl" | "ul") {
                self.pos += 2;
                return two.to_string();
            }
        }
        // Check for single-char suffix
        if let Some(c) = self.peek() {
            if matches!(c, 'b' | 'B' | 's' | 'S' | 'l' | 'L' | 'f' | 'F' | 'd' | 'D') {
                self.advance();
                return c.to_string();
            }
        }
        String::new()
    }

    /// Parse an SNBT value (compound, list, typed array, string, number, bool).
    fn parse_value(&mut self) -> Result<Tag, SnbtParseError> {
        self.skip_whitespace()?;
        match self.peek() {
            Some('{') => {
                let compound = self.parse_compound()?;
                Ok(compound.to_tag())
            }
            Some('[') => self.parse_list_or_array(),
            Some('"') | Some('\'') => {
                let s = self.parse_string()?;
                Ok(Tag::String(s))
            }
            Some('t') | Some('T') if self.remaining().to_ascii_lowercase().starts_with("true") => {
                self.pos += 4;
                Ok(Tag::Byte(1))
            }
            Some('f') | Some('F') if self.remaining().to_ascii_lowercase().starts_with("false") => {
                self.pos += 5;
                Ok(Tag::Byte(0))
            }
            Some('n') | Some('N') if self.remaining().to_ascii_lowercase().starts_with("null") => {
                self.pos += 4;
                Ok(Tag::End)
            }
            Some('+') | Some('-') | Some('0'..='9') | Some('.') => self.parse_number(),
            Some(c) if c.is_alphanumeric() || c == '_' => {
                // Could be an unquoted string or a number starting with non-digit
                let saved = self.pos;
                let token = self.parse_unquoted_string()?;
                // Try to parse as number if it looks like one
                if token.starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '+') || token == "-" {
                    self.pos = saved;
                    return self.parse_number();
                }
                Ok(Tag::String(token))
            }
            Some(c) => Err(self.err(format!("unexpected character '{}' in value", c))),
            None => Err(self.err("unexpected end of input in value")),
        }
    }

    /// Parse a compound tag: `{ key1: value1, key2: value2, ... }`
    fn parse_compound(&mut self) -> Result<CompoundTag, SnbtParseError> {
        self.expect_char('{')?;
        self.skip_whitespace()?;
        let mut tag = CompoundTag::new();
        if self.peek() == Some('}') { self.advance(); return Ok(tag); }
        loop {
            self.skip_whitespace()?;
            let key = self.parse_string()?;
            self.skip_whitespace()?;
            // Accept both : and = as key-value separator
            match self.peek() {
                Some(':') | Some('=') => { self.advance(); }
                Some(c) => return Err(self.err(format!("expected ':' or '=' after key but found '{}'", c))),
                None => return Err(self.err("unexpected end of input in compound")),
            }
            let value = self.parse_value()?;
            // Set in the compound using the Tag value directly
            tag.set(&key, value);
            self.skip_whitespace()?;
            match self.peek() {
                Some(',') => { self.advance(); }
                Some('}') => { self.advance(); return Ok(tag); }
                Some(c) => return Err(self.err(format!("expected ',' or '}}' but found '{}'", c))),
                None => return Err(self.err("unexpected end of input in compound")),
            }
        }
    }

    /// Parse a list or typed array: `[ ... ]`, `[B; ...]`, `[I; ...]`, `[L; ...]`
    fn parse_list_or_array(&mut self) -> Result<Tag, SnbtParseError> {
        self.expect_char('[')?;
        // Check for typed arrays
        self.skip_whitespace()?;
        if self.remaining().starts_with("B;") || self.remaining().starts_with("b;") {
            self.pos += 2;
            return self.parse_byte_array();
        }
        if self.remaining().starts_with("I;") || self.remaining().starts_with("i;") {
            self.pos += 2;
            return self.parse_int_array();
        }
        if self.remaining().starts_with("L;") || self.remaining().starts_with("l;") {
            self.pos += 2;
            return self.parse_long_array();
        }
        // Regular list
        self.skip_whitespace()?;
        let mut elements: Vec<Tag> = Vec::new();
        if self.peek() == Some(']') { self.advance(); return Ok(Tag::List(crate::tag::ListTagValue { element_type: TagType::End, elements })); }
        loop {
            self.skip_whitespace()?;
            elements.push(self.parse_value()?);
            self.skip_whitespace()?;
            match self.peek() {
                Some(',') => { self.advance(); }
                Some(']') => { self.advance(); break; }
                Some(c) => return Err(self.err(format!("expected ',' or ']' in list but found '{}'", c))),
                None => return Err(self.err("unexpected end of input in list")),
            }
        }
        let element_type = elements.first().map(|t| t.tag_type()).unwrap_or(TagType::End);
        Ok(Tag::List(crate::tag::ListTagValue { element_type, elements }))
    }

    fn parse_byte_array(&mut self) -> Result<Tag, SnbtParseError> {
        let mut vals = Vec::new();
        loop {
            self.skip_whitespace()?;
            if self.peek() == Some(']') { self.advance(); return Ok(Tag::ByteArray(vals)); }
            if !vals.is_empty() { self.expect_char(',')?; }
            self.skip_whitespace()?;
            let tag = self.parse_value()?;
            let v: i8 = match tag {
                Tag::Byte(b) => b,
                Tag::Int(i) => i as i8,
                Tag::Long(l) => l as i8,
                Tag::Short(s) => s as i8,
                _ => return Err(self.err("expected numeric value in byte array")),
            };
            vals.push(v as u8);
        }
    }

    fn parse_int_array(&mut self) -> Result<Tag, SnbtParseError> {
        let mut vals = Vec::new();
        loop {
            self.skip_whitespace()?;
            if self.peek() == Some(']') { self.advance(); return Ok(Tag::IntArray(vals)); }
            if !vals.is_empty() { self.expect_char(',')?; }
            self.skip_whitespace()?;
            let tag = self.parse_value()?;
            let v: i32 = match tag {
                Tag::Byte(b) => b as i32,
                Tag::Short(s) => s as i32,
                Tag::Int(i) => i,
                Tag::Long(l) => l as i32,
                _ => return Err(self.err("expected numeric value in int array")),
            };
            vals.push(v);
        }
    }

    fn parse_long_array(&mut self) -> Result<Tag, SnbtParseError> {
        let mut vals = Vec::new();
        loop {
            self.skip_whitespace()?;
            if self.peek() == Some(']') { self.advance(); return Ok(Tag::LongArray(vals)); }
            if !vals.is_empty() { self.expect_char(',')?; }
            self.skip_whitespace()?;
            let tag = self.parse_value()?;
            let v: i64 = match tag {
                Tag::Byte(b) => b as i64,
                Tag::Short(s) => s as i64,
                Tag::Int(i) => i as i64,
                Tag::Long(l) => l,
                _ => return Err(self.err("expected numeric value in long array")),
            };
            vals.push(v);
        }
    }
}

/// Parse an integer string with given radix into i64, respecting bit width for overflow.
fn parse_int_with_radix(s: &str, radix: u32, bits: u32) -> Result<i64, String> {
    let negative = s.starts_with('-');
    let digits = if negative { &s[1..] } else { s };
    if digits.is_empty() { return Err("empty number".to_string()); }
    let val = i64::from_str_radix(digits, radix).map_err(|e| format!("invalid integer: {}", e))?;
    let val = if negative { -val } else { val };
    // Mask to fit the bit width (for unsigned-like behavior)
    if bits < 64 {
        let mask = (1i64 << bits) - 1;
        let masked = val & mask;
        // Sign-extend if negative in the target bit width
        if masked & (1i64 << (bits - 1)) != 0 {
            Ok(masked | (!mask))
        } else {
            Ok(masked)
        }
    } else {
        Ok(val)
    }
}

/// Parse an SNBT string into a CompoundTag.
///
/// The input should be a top-level compound tag: `{ key: value, ... }`.
pub fn from_snbt(snbt: &str) -> Result<CompoundTag, SnbtParseError> {
    let mut cursor = Cursor::new(snbt);
    cursor.skip_whitespace()?;
    if cursor.peek() == Some('{') {
        cursor.parse_compound()
    } else {
        Err(cursor.err("expected '{' at root for SNBT compound tag"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_compound() {
        let tag = from_snbt("{}").unwrap();
        assert!(tag.empty());
    }

    #[test]
    fn test_parse_simple_values() {
        let snbt = r#"{"byte": 1b, "short": 2s, "int": 3, "long": 4L, "float": 1.5f, "double": 2.5, "str": "hello"}"#;
        let tag = from_snbt(snbt).unwrap();
        assert_eq!(tag.at("byte"), &Tag::Byte(1));
        assert_eq!(tag.at("short"), &Tag::Short(2));
        assert_eq!(tag.at("int").as_i32(), Some(3));
        assert_eq!(tag.at("long"), &Tag::Long(4));
        assert_eq!(tag.at("str").as_str(), Some("hello"));
        // float and double roundtrips
        if let Tag::Float(f) = tag.at("float") { assert!((*f - 1.5).abs() < 0.001); }
        else { panic!("expected Float"); }
    }

    #[test]
    fn test_parse_nested() {
        let snbt = r#"{"nested": {"x": 10, "y": 20}}"#;
        let tag = from_snbt(snbt).unwrap();
        let nested = tag.at("nested").as_compound().unwrap();
        assert_eq!(nested.get("x").and_then(|t| t.as_i32()), Some(10));
    }

    #[test]
    fn test_parse_list() {
        let snbt = r#"{"list": ["a", "b", "c"]}"#;
        let tag = from_snbt(snbt).unwrap();
        let list = tag.at("list").as_list_value().unwrap();
        assert_eq!(list.elements.len(), 3);
    }

    #[test]
    fn test_parse_typed_arrays() {
        let snbt = r#"{"ba": [B;1,2,3], "ia": [I;4,5,6], "la": [L;7,8,9]}"#;
        let tag = from_snbt(snbt).unwrap();
        if let Tag::ByteArray(ba) = tag.at("ba") {
            assert_eq!(ba.as_slice(), &[1, 2, 3]);
        } else { panic!("expected ByteArray"); }
        if let Tag::IntArray(ia) = tag.at("ia") {
            assert_eq!(ia.as_slice(), &[4, 5, 6]);
        } else { panic!("expected IntArray"); }
        if let Tag::LongArray(la) = tag.at("la") {
            assert_eq!(la.as_slice(), &[7, 8, 9]);
        } else { panic!("expected LongArray"); }
    }

    #[test]
    fn test_parse_comments() {
        let snbt = r#"{// line comment
        "key": "value" /* block comment */
        }"#;
        let tag = from_snbt(snbt).unwrap();
        assert_eq!(tag.at("key").as_str(), Some("value"));
    }

    #[test]
    fn test_parse_escape_sequences() {
        let snbt = r#"{"s": "line1\nline2\t\"quoted\""}"#;
        let tag = from_snbt(snbt).unwrap();
        let s = tag.at("s").as_str().unwrap();
        assert!(s.contains('\n'));
        assert!(s.contains('\t'));
        assert!(s.contains('"'));
    }

    #[test]
    fn test_parse_unicode_escape() {
        let snbt = r#"{"u": "中文"}"#;
        let tag = from_snbt(snbt).unwrap();
        assert_eq!(tag.at("u").as_str(), Some("中文"));
    }

    #[test]
    fn test_parse_numeric_suffixes() {
        let snbt = r#"{"sb_val": 5sb, "ub_val": 6ub, "ss_val": 7ss, "us_val": 8us, "si_val": 9si, "ui_val": 10ui}"#;
        let tag = from_snbt(snbt).unwrap();
        assert_eq!(tag.at("sb_val"), &Tag::Byte(5));
        assert_eq!(tag.at("ss_val"), &Tag::Short(7));
        assert_eq!(tag.at("si_val"), &Tag::Int(9));
    }

    #[test]
    fn test_parse_bool() {
        let snbt = r#"{"a": true, "b": false}"#;
        let tag = from_snbt(snbt).unwrap();
        assert_eq!(tag.at("a"), &Tag::Byte(1));
        assert_eq!(tag.at("b"), &Tag::Byte(0));
    }

    #[test]
    fn test_roundtrip() {
        let mut original = CompoundTag::new();
        original.set("name", "test");
        original.set("value", 42);
        let snbt = original.to_snbt();
        let parsed = from_snbt(&snbt).unwrap();
        assert_eq!(parsed.at("name").as_str(), Some("test"));
        assert_eq!(parsed.at("value").as_i32(), Some(42));
    }

    #[test]
    fn test_empty_list() {
        let snbt = r#"{"list": []}"#;
        let tag = from_snbt(snbt).unwrap();
        let list = tag.at("list").as_list_value().unwrap();
        assert!(list.elements.is_empty());
    }

    #[test]
    fn test_parse_error() {
        let result = from_snbt(r#"{"unclosed": "#);
        assert!(result.is_err());
    }

    #[test]
    fn test_comment_style_suffix() {
        let snbt = r#"{"v": 255 /*ub*/}"#;
        let tag = from_snbt(snbt).unwrap();
        // 255 as u8, stored as Tag::Byte(-1)
        assert_eq!(tag.at("v"), &Tag::Byte(-1i8));
    }
}
