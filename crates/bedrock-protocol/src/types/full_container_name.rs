use bedrock_common::{BedrockRead, BedrockSerializable, BinaryStreamError, BedrockWrite};

/// Container name with optional dynamic slot ID.
#[derive(Debug, Clone, PartialEq)]
pub struct FullContainerName {
    pub container_enum: u8,
    pub dynamic_slot: Option<u32>,
}

impl Default for FullContainerName {
    fn default() -> Self {
        Self { container_enum: 0, dynamic_slot: None }
    }
}

impl BedrockSerializable for FullContainerName {
    fn write_to(&self, stream: &mut dyn BedrockWrite) -> Result<(), BinaryStreamError> {
        stream.write_u8(self.container_enum)?;
        stream.write_bool(self.dynamic_slot.is_some())?;
        if let Some(slot) = self.dynamic_slot {
            stream.write_u32(slot)?;
        }
        Ok(())
    }

    fn read_from(stream: &mut dyn BedrockRead) -> Result<Self, BinaryStreamError> {
        let container_enum = stream.read_u8()?;
        let dynamic_slot = if stream.read_bool()? {
            Some(stream.read_u32()?)
        } else {
            None
        };
        Ok(Self { container_enum, dynamic_slot })
    }
}
