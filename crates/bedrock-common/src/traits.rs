use crate::BinaryStreamError;

/// Trait for types that can be serialized to and deserialized from a Bedrock binary stream.
///
/// This is the core serialization abstraction used by all packet and type definitions.
/// It mirrors the symmetric `write(stream)` / `read(stream)` pattern from the C++ bstream
/// library and the existing Python bedrock-protocol-packets codebase.
///
/// # Symmetry
///
/// Implementations MUST ensure that a value written with `write_to` can be fully recovered
/// by `read_from` using the same stream configuration (endianness). The order of field
/// reads must exactly match the order of field writes.
///
/// # Errors
///
/// Both methods return `BinaryStreamError` to handle buffer overflows, invalid data,
/// and other I/O-level failures without panicking.
pub trait BedrockSerializable {
    /// Serialize `self` into the binary stream.
    ///
    /// Fields must be written in the exact order defined by the protocol specification.
    fn write_to(&self, stream: &mut dyn BedrockWrite) -> Result<(), BinaryStreamError>;

    /// Deserialize from the binary stream, consuming the minimum number of bytes needed.
    fn read_from(stream: &mut dyn BedrockRead) -> Result<Self, BinaryStreamError>
    where Self: Sized;
}

/// Read-only operations on a binary stream.
///
/// This trait enables polymorphic access to read operations. The primary concrete
/// implementation is `ReadOnlyBinaryStream` in the `bedrock-binary-stream` crate.
pub trait BedrockRead {
    fn read_bool(&mut self) -> Result<bool, BinaryStreamError>;
    fn read_u8(&mut self) -> Result<u8, BinaryStreamError>;
    fn read_i16(&mut self) -> Result<i16, BinaryStreamError>;
    fn read_u16(&mut self) -> Result<u16, BinaryStreamError>;
    fn read_i32(&mut self) -> Result<i32, BinaryStreamError>;
    fn read_u32(&mut self) -> Result<u32, BinaryStreamError>;
    fn read_i64(&mut self) -> Result<i64, BinaryStreamError>;
    fn read_u64(&mut self) -> Result<u64, BinaryStreamError>;
    fn read_f32(&mut self) -> Result<f32, BinaryStreamError>;
    fn read_f64(&mut self) -> Result<f64, BinaryStreamError>;
    fn read_i32_be(&mut self) -> Result<i32, BinaryStreamError>;
    fn read_u32_be(&mut self) -> Result<u32, BinaryStreamError>;
    fn read_u24(&mut self) -> Result<u32, BinaryStreamError>;
    fn read_varint(&mut self) -> Result<i32, BinaryStreamError>;
    fn read_varint64(&mut self) -> Result<i64, BinaryStreamError>;
    fn read_unsigned_varint(&mut self) -> Result<u32, BinaryStreamError>;
    fn read_unsigned_varint64(&mut self) -> Result<u64, BinaryStreamError>;
    fn read_normalized_f32(&mut self) -> Result<f32, BinaryStreamError>;
    fn read_string(&mut self) -> Result<String, BinaryStreamError>;
    fn read_short_string(&mut self) -> Result<String, BinaryStreamError>;
    fn read_long_string(&mut self) -> Result<String, BinaryStreamError>;
    fn read_raw_bytes(&mut self, len: usize) -> Result<Vec<u8>, BinaryStreamError>;
    fn read_remaining(&mut self) -> Result<Vec<u8>, BinaryStreamError>;
}

/// Write-only operations on a binary stream.
///
/// This trait enables polymorphic access to write operations. The primary concrete
/// implementation is `BinaryStream` in the `bedrock-binary-stream` crate.
pub trait BedrockWrite {
    fn write_bool(&mut self, value: bool) -> Result<(), BinaryStreamError>;
    fn write_u8(&mut self, value: u8) -> Result<(), BinaryStreamError>;
    fn write_i16(&mut self, value: i16) -> Result<(), BinaryStreamError>;
    fn write_u16(&mut self, value: u16) -> Result<(), BinaryStreamError>;
    fn write_i32(&mut self, value: i32) -> Result<(), BinaryStreamError>;
    fn write_u32(&mut self, value: u32) -> Result<(), BinaryStreamError>;
    fn write_i64(&mut self, value: i64) -> Result<(), BinaryStreamError>;
    fn write_u64(&mut self, value: u64) -> Result<(), BinaryStreamError>;
    fn write_f32(&mut self, value: f32) -> Result<(), BinaryStreamError>;
    fn write_f64(&mut self, value: f64) -> Result<(), BinaryStreamError>;
    fn write_i32_be(&mut self, value: i32) -> Result<(), BinaryStreamError>;
    fn write_u32_be(&mut self, value: u32) -> Result<(), BinaryStreamError>;
    fn write_u24(&mut self, value: u32) -> Result<(), BinaryStreamError>;
    fn write_varint(&mut self, value: i32) -> Result<(), BinaryStreamError>;
    fn write_varint64(&mut self, value: i64) -> Result<(), BinaryStreamError>;
    fn write_unsigned_varint(&mut self, value: u32) -> Result<(), BinaryStreamError>;
    fn write_unsigned_varint64(&mut self, value: u64) -> Result<(), BinaryStreamError>;
    fn write_normalized_f32(&mut self, value: f32) -> Result<(), BinaryStreamError>;
    fn write_string(&mut self, value: &str) -> Result<(), BinaryStreamError>;
    fn write_short_string(&mut self, value: &str) -> Result<(), BinaryStreamError>;
    fn write_long_string(&mut self, value: &str) -> Result<(), BinaryStreamError>;
    fn write_raw_bytes(&mut self, value: &[u8]) -> Result<(), BinaryStreamError>;
}
