use bedrock_common::{BedrockRead, BedrockSerializable, BinaryStreamError, BedrockWrite};

/// A block position using signed varint-encoded coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Default for BlockPos {
    fn default() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }
}

impl BedrockSerializable for BlockPos {
    fn write_to(&self, stream: &mut dyn BedrockWrite) -> Result<(), BinaryStreamError> {
        stream.write_varint(self.x)?;
        stream.write_varint(self.y)?;
        stream.write_varint(self.z)
    }

    fn read_from(stream: &mut dyn BedrockRead) -> Result<Self, BinaryStreamError> {
        Ok(Self {
            x: stream.read_varint()?,
            y: stream.read_varint()?,
            z: stream.read_varint()?,
        })
    }
}
