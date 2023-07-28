use crate::terrain::color::AtomColor;

use super::id::MappedToId;

#[derive(Debug, Clone)]
pub struct Element {
    pub color: AtomColor,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            color: AtomColor::WHITE,
        }
    }
}

impl MappedToId for Element {
    type Id = u8;
}
