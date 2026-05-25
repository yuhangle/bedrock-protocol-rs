use bedrock_common::{BedrockRead, BedrockSerializable, BinaryStreamError, BedrockWrite};
use crate::packet::Packet;
use crate::MinecraftPacketIds;

/// A fallback packet for unknown/unimplemented packet IDs.
///
/// Stores the raw payload bytes so that unrecognized packets can still be
/// forwarded or inspected without data loss.
#[derive(Debug, Clone, PartialEq)]
pub struct UnimplementedPacket {
    pub packet_id: MinecraftPacketIds,
    pub payload: Vec<u8>,
}

impl UnimplementedPacket {
    pub fn new(packet_id: impl Into<i32>) -> Self {
        Self {
            packet_id: MinecraftPacketIds::from_i32(packet_id.into())
                .unwrap_or(MinecraftPacketIds::EndId),
            payload: Vec::new(),
        }
    }
}

impl Packet for UnimplementedPacket {
    fn packet_id(&self) -> MinecraftPacketIds {
        self.packet_id
    }

    fn packet_name(&self) -> &'static str {
        "UnimplementedPacket"
    }
}

impl BedrockSerializable for UnimplementedPacket {
    fn write_to(&self, stream: &mut dyn BedrockWrite) -> Result<(), BinaryStreamError> {
        stream.write_raw_bytes(&self.payload)
    }

    fn read_from(stream: &mut dyn BedrockRead) -> Result<Self, BinaryStreamError> {
        Ok(Self {
            packet_id: MinecraftPacketIds::EndId, // Will be set by the caller
            payload: stream.read_remaining()?,
        })
    }
}
