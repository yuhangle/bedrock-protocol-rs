use bedrock_common::{BedrockSerializable, BinaryStreamError};
use bedrock_binary_stream::{BinaryStream, ReadOnlyBinaryStream};

use crate::MinecraftPacketIds;

/// The core trait for all Minecraft Bedrock Edition packets.
///
/// Each packet type must implement:
/// - `packet_id()` — the numeric packet ID
/// - `packet_name()` — the human-readable name
///
/// Serialization is provided via `BedrockSerializable` (the `read_from`/`write_to` methods).
/// The `serialize()` and `deserialize()` helper methods handle stream lifecycle.
pub trait Packet: BedrockSerializable {
    /// Return the packet's numeric ID.
    fn packet_id(&self) -> MinecraftPacketIds;

    /// Return the human-readable packet name.
    fn packet_name(&self) -> &'static str;

    /// Serialize the packet to a byte vector.
    ///
    /// Creates a new `BinaryStream`, calls `write_to`, and returns the buffer.
    fn serialize(&self) -> Result<Vec<u8>, BinaryStreamError> {
        let mut stream = BinaryStream::new(false);
        self.write_to(&mut stream)?;
        Ok(stream.into_data())
    }

    /// Deserialize the packet from a byte slice.
    ///
    /// Creates a `ReadOnlyBinaryStream` from the data and calls `read_from`.
    fn deserialize(&mut self, data: &[u8]) -> Result<(), BinaryStreamError>
    where Self: Sized {
        let mut stream = ReadOnlyBinaryStream::new(data, false);
        *self = Self::read_from(&mut stream)?;
        Ok(())
    }

}
