use bedrock_common::{BedrockRead, BedrockSerializable, BinaryStreamError, BedrockWrite};

/// A Minecraft UUID, stored as two `u64` halves (high, low).
///
/// Serialized as two unsigned int64 values in little-endian order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Uuid {
    pub high: u64,
    pub low: u64,
}

impl Default for Uuid {
    fn default() -> Self {
        Self { high: 0, low: 0 }
    }
}

impl Uuid {
    /// Create a UUID from a hyphenated string (e.g., `"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"`).
    pub fn from_string(uuid: &str) -> Result<Self, &'static str> {
        let hex: String = uuid.chars().filter(|c| c.is_ascii_hexdigit()).collect();
        if hex.len() != 32 {
            return Err("invalid UUID format: expected 32 hex digits");
        }
        let high = u64::from_str_radix(&hex[0..16], 16)
            .map_err(|_| "invalid hex in UUID high")?;
        let low = u64::from_str_radix(&hex[16..32], 16)
            .map_err(|_| "invalid hex in UUID low")?;
        Ok(Self { high, low })
    }

    /// Format as a hyphenated UUID string.
    pub fn to_string(&self) -> String {
        let h = format!("{:016x}", self.high);
        let l = format!("{:016x}", self.low);
        format!(
            "{}-{}-{}-{}-{}",
            &h[0..8],
            &h[8..12],
            &h[12..16],
            &l[0..4],
            &l[4..16]
        )
    }
}

impl BedrockSerializable for Uuid {
    fn write_to(&self, stream: &mut dyn BedrockWrite) -> Result<(), BinaryStreamError> {
        stream.write_u64(self.high)?;
        stream.write_u64(self.low)
    }

    fn read_from(stream: &mut dyn BedrockRead) -> Result<Self, BinaryStreamError> {
        Ok(Self {
            high: stream.read_u64()?,
            low: stream.read_u64()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_roundtrip() {
        let uuid = Uuid::from_string("12345678-1234-5678-1234-567812345678").unwrap();
        let s = uuid.to_string();
        assert_eq!(s, "12345678-1234-5678-1234-567812345678");
    }

    #[test]
    fn test_uuid_default() {
        let uuid = Uuid::default();
        assert_eq!(uuid.high, 0);
        assert_eq!(uuid.low, 0);
    }
}
