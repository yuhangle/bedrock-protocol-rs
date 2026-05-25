mod packet;
mod unimplemented;

pub mod types;

pub use packet::Packet;
pub use unimplemented::UnimplementedPacket;

pub use bedrock_common::{BedrockRead, BedrockSerializable, BedrockWrite, BinaryStreamError};
pub use bedrock_binary_stream::{BinaryStream, ReadOnlyBinaryStream};

#[cfg(not(feature = "generated"))]
mod ids;
#[cfg(not(feature = "generated"))]
pub use ids::MinecraftPacketIds;

#[cfg(feature = "generated")]
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
#[cfg(feature = "generated")]
pub use gen_packet_ids::MinecraftPacketIds;
