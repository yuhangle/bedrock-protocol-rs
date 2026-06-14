use bedrock_common::{BedrockRead, BedrockSerializable, BinaryStreamError, BedrockWrite};

// -----------------------------------------------------------------------
// ItemStackRequestActionType
// -----------------------------------------------------------------------

/// Types of item stack request actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ItemStackRequestActionType {
    Invalid = 0,
    Take = 1,
    Place = 2,
    Swap = 3,
    Drop = 4,
    Destroy = 5,
    Consume = 6,
    Create = 7,
    LabTableCombine = 8,
    BeaconPayment = 9,
    MineBlock = 10,
    CraftRecipe = 11,
    CraftRecipeAuto = 12,
    CraftCreative = 13,
    CraftRecipeOptional = 14,
    CraftNonImplemented_Deprecated = 15,
    CraftResults_Deprecated = 16,
}

impl ItemStackRequestActionType {
    pub fn from_i32(v: i32) -> Option<Self> {
        Some(match v {
            0 => Self::Invalid, 1 => Self::Take, 2 => Self::Place,
            3 => Self::Swap, 4 => Self::Drop, 5 => Self::Destroy,
            6 => Self::Consume, 7 => Self::Create, 8 => Self::LabTableCombine,
            9 => Self::BeaconPayment, 10 => Self::MineBlock, 11 => Self::CraftRecipe,
            12 => Self::CraftRecipeAuto, 13 => Self::CraftCreative,
            14 => Self::CraftRecipeOptional, 15 => Self::CraftNonImplemented_Deprecated,
            16 => Self::CraftResults_Deprecated,
            _ => return None,
        })
    }
}

// -----------------------------------------------------------------------
// ItemStackRequestSlotInfo
// -----------------------------------------------------------------------

use crate::types::FullContainerName;

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStackRequestSlotInfo {
    pub container: FullContainerName,
    pub slot: u8,
    pub net_id: i32,
}

impl Default for ItemStackRequestSlotInfo {
    fn default() -> Self {
        Self {
            container: FullContainerName::default(),
            slot: 0,
            net_id: 0,
        }
    }
}

impl BedrockSerializable for ItemStackRequestSlotInfo {
    fn write_to(&self, stream: &mut dyn BedrockWrite) -> Result<(), BinaryStreamError> {
        self.container.write_to(stream)?;
        stream.write_u8(self.slot)?;
        stream.write_varint(self.net_id)
    }

    fn read_from(stream: &mut dyn BedrockRead) -> Result<Self, BinaryStreamError> {
        Ok(Self {
            container: FullContainerName::read_from(stream)?,
            slot: stream.read_u8()?,
            net_id: stream.read_varint()?,
        })
    }
}

// -----------------------------------------------------------------------
// ItemStackRequestActionTransferBase
// -----------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStackRequestActionTransferBase {
    pub amount: u8,
    pub source: ItemStackRequestSlotInfo,
    pub destination: ItemStackRequestSlotInfo,
}

impl Default for ItemStackRequestActionTransferBase {
    fn default() -> Self {
        Self {
            amount: 0,
            source: ItemStackRequestSlotInfo::default(),
            destination: ItemStackRequestSlotInfo::default(),
        }
    }
}

impl BedrockSerializable for ItemStackRequestActionTransferBase {
    fn write_to(&self, stream: &mut dyn BedrockWrite) -> Result<(), BinaryStreamError> {
        stream.write_u8(self.amount)?;
        self.source.write_to(stream)?;
        self.destination.write_to(stream)
    }

    fn read_from(stream: &mut dyn BedrockRead) -> Result<Self, BinaryStreamError> {
        Ok(Self {
            amount: stream.read_u8()?,
            source: ItemStackRequestSlotInfo::read_from(stream)?,
            destination: ItemStackRequestSlotInfo::read_from(stream)?,
        })
    }
}

// -----------------------------------------------------------------------
// ItemStackRequestAction
// -----------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStackRequestAction {
    pub action_type: ItemStackRequestActionType,
    pub action_data: Option<ItemStackRequestActionTransferBase>,
}

impl Default for ItemStackRequestAction {
    fn default() -> Self {
        Self {
            action_type: ItemStackRequestActionType::Invalid,
            action_data: None,
        }
    }
}

impl BedrockSerializable for ItemStackRequestAction {
    fn write_to(&self, stream: &mut dyn BedrockWrite) -> Result<(), BinaryStreamError> {
        stream.write_u8(self.action_type as u8)?;
        if let Some(ref data) = self.action_data {
            data.write_to(stream)?;
        }
        Ok(())
    }

    fn read_from(stream: &mut dyn BedrockRead) -> Result<Self, BinaryStreamError> {
        let action_type_val = stream.read_u8()?;
        let action_type = ItemStackRequestActionType::from_i32(action_type_val as i32)
            .unwrap_or(ItemStackRequestActionType::Invalid);

        let action_data = match action_type {
            // Take/Place: amount + source + destination
            ItemStackRequestActionType::Take | ItemStackRequestActionType::Place => {
                Some(ItemStackRequestActionTransferBase::read_from(stream)?)
            }
            // Swap: source + destination (no amount)
            ItemStackRequestActionType::Swap => {
                let source = ItemStackRequestSlotInfo::read_from(stream)?;
                let destination = ItemStackRequestSlotInfo::read_from(stream)?;
                Some(ItemStackRequestActionTransferBase {
                    amount: 0,
                    source,
                    destination,
                })
            }
            // Drop/Destroy: source + amount + count_id (varint)
            ItemStackRequestActionType::Drop | ItemStackRequestActionType::Destroy => {
                let source = ItemStackRequestSlotInfo::read_from(stream)?;
                let amount = stream.read_u8()?;
                let _count_id = stream.read_varint()?;
                Some(ItemStackRequestActionTransferBase {
                    amount,
                    source,
                    destination: ItemStackRequestSlotInfo::default(),
                })
            }
            // Create: amount
            ItemStackRequestActionType::Create => {
                let amount = stream.read_u8()?;
                Some(ItemStackRequestActionTransferBase {
                    amount,
                    source: ItemStackRequestSlotInfo::default(),
                    destination: ItemStackRequestSlotInfo::default(),
                })
            }
            // Other action types: no additional data
            _ => None,
        };

        Ok(Self { action_type, action_data })
    }
}

// -----------------------------------------------------------------------
// ItemStackRequestData
// -----------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStackRequestData {
    pub client_request_id: i32,
    pub request_actions: Vec<ItemStackRequestAction>,
}

impl Default for ItemStackRequestData {
    fn default() -> Self {
        Self {
            client_request_id: 0,
            request_actions: Vec::new(),
        }
    }
}

impl BedrockSerializable for ItemStackRequestData {
    fn write_to(&self, stream: &mut dyn BedrockWrite) -> Result<(), BinaryStreamError> {
        stream.write_varint(self.client_request_id)?;
        stream.write_unsigned_varint(self.request_actions.len() as u32)?;
        for action in &self.request_actions {
            action.write_to(stream)?;
        }
        Ok(())
    }

    fn read_from(stream: &mut dyn BedrockRead) -> Result<Self, BinaryStreamError> {
        let client_request_id = stream.read_varint()?;
        let count = stream.read_unsigned_varint()?;
        let mut request_actions = Vec::with_capacity(count as usize);
        for _ in 0..count {
            request_actions.push(ItemStackRequestAction::read_from(stream)?);
        }
        Ok(Self { client_request_id, request_actions })
    }
}

// -----------------------------------------------------------------------
// ItemStackRequest (wrapper)
// -----------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStackRequest {
    pub request_data: Vec<ItemStackRequestData>,
}

impl Default for ItemStackRequest {
    fn default() -> Self {
        Self { request_data: Vec::new() }
    }
}

impl BedrockSerializable for ItemStackRequest {
    fn write_to(&self, stream: &mut dyn BedrockWrite) -> Result<(), BinaryStreamError> {
        stream.write_unsigned_varint(self.request_data.len() as u32)?;
        for req in &self.request_data {
            req.write_to(stream)?;
        }
        Ok(())
    }

    fn read_from(stream: &mut dyn BedrockRead) -> Result<Self, BinaryStreamError> {
        let count = stream.read_unsigned_varint()?;
        let mut request_data = Vec::with_capacity(count as usize);
        for _ in 0..count {
            request_data.push(ItemStackRequestData::read_from(stream)?);
        }
        Ok(Self { request_data })
    }
}
