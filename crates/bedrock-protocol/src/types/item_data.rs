use bedrock_common::{BedrockRead, BedrockSerializable, BinaryStreamError, BedrockWrite};

/// Item registry data from ItemRegistryPacket.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemData {
    pub item_name: String,
    pub item_id: i16,
    pub is_component_based: bool,
    pub item_version: i32,
    /// Component NBT data as raw bytes (should be parsed as NBT CompoundTag).
    pub component_data: Vec<u8>,
}

impl Default for ItemData {
    fn default() -> Self {
        Self {
            item_name: String::new(),
            item_id: 0,
            is_component_based: false,
            item_version: 0,
            component_data: Vec::new(),
        }
    }
}

impl BedrockSerializable for ItemData {
    fn write_to(&self, stream: &mut dyn BedrockWrite) -> Result<(), BinaryStreamError> {
        stream.write_string(&self.item_name)?;
        stream.write_i16(self.item_id)?;
        stream.write_bool(self.is_component_based)?;
        stream.write_varint(self.item_version)?;
        stream.write_raw_bytes(&self.component_data)
    }

    fn read_from(stream: &mut dyn BedrockRead) -> Result<Self, BinaryStreamError> {
        Ok(Self {
            item_name: stream.read_string()?,
            item_id: stream.read_i16()?,
            is_component_based: stream.read_bool()?,
            item_version: stream.read_varint()?,
            component_data: stream.read_remaining()?, // NBT data follows
        })
    }
}
