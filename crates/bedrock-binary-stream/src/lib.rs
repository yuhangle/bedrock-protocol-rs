//! Binary stream serialization core for the Bedrock protocol.
//!
//! Provides `ReadOnlyBinaryStream` (read-only, borrows or owns its buffer) and
//! `BinaryStream` (writable, owns its buffer) with support for all types used
//! by the Minecraft Bedrock Edition protocol: fixed-width integers, floats,
//! varints, strings, NBT tags, and normalization.

mod read;
mod write;

pub use bedrock_common::{BedrockRead, BedrockWrite};
pub use read::ReadOnlyBinaryStream;
pub use write::BinaryStream;
