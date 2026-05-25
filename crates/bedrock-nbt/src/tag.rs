use std::collections::HashMap;

/// All possible NBT tag types.
#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List(ListTagValue),
    Compound(HashMap<String, Tag>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

/// The value contained in a ListTag.
#[derive(Debug, Clone, PartialEq)]
pub struct ListTagValue {
    pub element_type: TagType,
    pub elements: Vec<Tag>,
}

/// NBT type identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagType {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

impl TagType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::End),
            1 => Some(Self::Byte),
            2 => Some(Self::Short),
            3 => Some(Self::Int),
            4 => Some(Self::Long),
            5 => Some(Self::Float),
            6 => Some(Self::Double),
            7 => Some(Self::ByteArray),
            8 => Some(Self::String),
            9 => Some(Self::List),
            10 => Some(Self::Compound),
            11 => Some(Self::IntArray),
            12 => Some(Self::LongArray),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

impl Tag {
    /// Return the TagType of this tag.
    pub fn tag_type(&self) -> TagType {
        match self {
            Tag::End => TagType::End,
            Tag::Byte(_) => TagType::Byte,
            Tag::Short(_) => TagType::Short,
            Tag::Int(_) => TagType::Int,
            Tag::Long(_) => TagType::Long,
            Tag::Float(_) => TagType::Float,
            Tag::Double(_) => TagType::Double,
            Tag::ByteArray(_) => TagType::ByteArray,
            Tag::String(_) => TagType::String,
            Tag::List(_) => TagType::List,
            Tag::Compound(_) => TagType::Compound,
            Tag::IntArray(_) => TagType::IntArray,
            Tag::LongArray(_) => TagType::LongArray,
        }
    }

    /// Convert a Rust value into a Tag.
    /// Supports: bool→Byte, i32→Int, i16→Short, i8→Byte, u8→Byte,
    /// f32→Float, f64→Double, String→StringTag, Vec<Tag>→List, HashMap→Compound.
    pub fn from_value<T: Into<Tag>>(value: T) -> Self {
        value.into()
    }

    /// Convenience accessor: extract i32 value from Int tag.
    pub fn as_i32(&self) -> Option<i32> {
        match self { Tag::Int(v) => Some(*v), _ => None }
    }

    /// Convenience accessor: extract &str from String tag.
    pub fn as_str(&self) -> Option<&str> {
        match self { Tag::String(v) => Some(v.as_str()), _ => None }
    }

    /// Convenience accessor: extract Compound HashMap.
    pub fn as_compound(&self) -> Option<&std::collections::HashMap<String, Tag>> {
        match self { Tag::Compound(m) => Some(m), _ => None }
    }

    /// Convenience accessor: extract ListTagValue.
    pub fn as_list_value(&self) -> Option<&ListTagValue> {
        match self { Tag::List(l) => Some(l), _ => None }
    }
}

// ── From impls for automatic type conversion ──

impl From<bool> for Tag {
    fn from(v: bool) -> Self { Tag::Byte(if v { 1 } else { 0 }) }
}

impl From<i8> for Tag {
    fn from(v: i8) -> Self { Tag::Byte(v) }
}

impl From<u8> for Tag {
    fn from(v: u8) -> Self { Tag::Byte(v as i8) }
}

impl From<i16> for Tag {
    fn from(v: i16) -> Self { Tag::Short(v) }
}

impl From<u16> for Tag {
    fn from(v: u16) -> Self { Tag::Int(v as i32) }
}

impl From<i32> for Tag {
    fn from(v: i32) -> Self { Tag::Int(v) }
}

impl From<u32> for Tag {
    fn from(v: u32) -> Self { Tag::Long(v as i64) }
}

impl From<i64> for Tag {
    fn from(v: i64) -> Self { Tag::Long(v) }
}

impl From<f32> for Tag {
    fn from(v: f32) -> Self { Tag::Float(v) }
}

impl From<f64> for Tag {
    fn from(v: f64) -> Self { Tag::Double(v) }
}

impl From<String> for Tag {
    fn from(v: String) -> Self { Tag::String(v) }
}

impl From<&str> for Tag {
    fn from(v: &str) -> Self { Tag::String(v.to_string()) }
}

impl From<Vec<u8>> for Tag {
    fn from(v: Vec<u8>) -> Self { Tag::ByteArray(v) }
}

impl From<Vec<i32>> for Tag {
    fn from(v: Vec<i32>) -> Self { Tag::IntArray(v) }
}

impl From<Vec<String>> for Tag {
    fn from(strings: Vec<String>) -> Self {
        let elements: Vec<Tag> = strings.into_iter().map(Tag::String).collect();
        Tag::List(ListTagValue {
            element_type: TagType::String,
            elements,
        })
    }
}

// Helper: convert HashMap<String, V> where V: Into<Tag> to Tag::Compound
impl<T: Into<Tag>> From<HashMap<String, T>> for Tag {
    fn from(map: HashMap<String, T>) -> Self {
        Tag::Compound(map.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}
