use bedrock_common::{BedrockRead, BedrockSerializable, BinaryStreamError, BedrockWrite};

/// A 3D floating-point vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Vec3 {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl BedrockSerializable for Vec3 {
    fn write_to(&self, stream: &mut dyn BedrockWrite) -> Result<(), BinaryStreamError> {
        stream.write_f32(self.x)?;
        stream.write_f32(self.y)?;
        stream.write_f32(self.z)
    }

    fn read_from(stream: &mut dyn BedrockRead) -> Result<Self, BinaryStreamError> {
        Ok(Self {
            x: stream.read_f32()?,
            y: stream.read_f32()?,
            z: stream.read_f32()?,
        })
    }
}
