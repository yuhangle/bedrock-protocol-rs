use std::fmt;

/// Error type for all binary stream operations.
///
/// Every read/write operation returns `Result<_, BinaryStreamError>` rather than
/// panicking. This ensures safe C FFI boundaries and enables graceful error recovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryStreamError {
    /// Read or write position exceeded the buffer bounds.
    Overflow {
        /// The position where the operation was attempted.
        position: usize,
        /// The total buffer size.
        size: usize,
    },

    /// The data in the buffer is structurally invalid.
    InvalidData {
        /// Description of what was expected vs. what was found.
        description: &'static str,
    },

    /// An unsupported or unrecognized value was encountered.
    UnsupportedValue {
        /// Details about the unsupported value.
        description: String,
    },

    /// An NBT (Named Binary Tag) serialization error occurred.
    NbtError {
        /// Description of the NBT error.
        description: String,
    },
}

impl fmt::Display for BinaryStreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow { position, size } => {
                write!(
                    f,
                    "buffer overflow: attempted access at position {} (buffer size: {})",
                    position, size
                )
            }
            Self::InvalidData { description } => {
                write!(f, "invalid data: {}", description)
            }
            Self::UnsupportedValue { description } => {
                write!(f, "unsupported value: {}", description)
            }
            Self::NbtError { description } => {
                write!(f, "NBT error: {}", description)
            }
        }
    }
}

impl std::error::Error for BinaryStreamError {}
