use crate::tag::{Tag, TagType, ListTagValue};

/// An ordered list of NBT tags.
///
/// Matches the RapidNBT ListTag API. All elements must have the same type.
/// The element type is set on first `append()` call.
#[derive(Debug, Clone, PartialEq)]
pub struct ListTag {
    element_type: TagType,
    elements: Vec<Tag>,
}

impl ListTag {
    /// Create an empty list tag. The element type will be determined
    /// by the first appended element.
    pub fn new() -> Self {
        Self {
            element_type: TagType::End, // placeholder until first append
            elements: Vec::new(),
        }
    }

    /// Append a value to the list. Auto-converts via `T: Into<Tag>`.
    /// All values must have the same TagType.
    pub fn append<T: Into<Tag>>(&mut self, value: T) {
        let tag = value.into();
        if self.elements.is_empty() {
            self.element_type = tag.tag_type();
        }
        // Validate type match
        debug_assert_eq!(
            tag.tag_type(), self.element_type,
            "ListTag: all elements must have the same type"
        );
        self.elements.push(tag);
    }

    /// Get an element by index.
    pub fn get(&self, index: usize) -> Option<&Tag> {
        self.elements.get(index)
    }

    /// Number of elements.
    pub fn size(&self) -> usize {
        self.elements.len()
    }

    /// Check if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// The element type of this list.
    pub fn element_type(&self) -> TagType {
        self.element_type
    }

    /// Get the inner elements.
    pub fn elements(&self) -> &[Tag] {
        &self.elements
    }

    /// Convert to internal Tag::List.
    pub fn to_tag(&self) -> Tag {
        Tag::List(ListTagValue {
            element_type: self.element_type,
            elements: self.elements.clone(),
        })
    }
}

impl Default for ListTag {
    fn default() -> Self { Self::new() }
}
