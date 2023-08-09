use bevy::prelude::Resource;

use crate::terrain::{color::AtomColor, Atom, JoinFace};

use super::id::{CreateInstanceWithId, IdMap, MappedToId};

#[derive(Debug, Clone)]
pub struct Element {
    pub color: AtomColor,
    pub join_face: JoinFace,
}

pub type ElementId = u8;

impl Default for Element {
    fn default() -> Self {
        Self {
            color: AtomColor::WHITE,
            join_face: JoinFace::SameAlpha,
        }
    }
}

impl MappedToId for Element {
    type Id = ElementId;

    fn create_map() -> IdMap<Self> {
        let mut map = IdMap::new();
        // Can't error bc map is empty
        let _ = map.insert(
            "Void",
            Element {
                color: AtomColor::INVISIBLE,
                ..Default::default()
            },
        );
        let _ = map.insert(
            "Air",
            Element {
                color: AtomColor::INVISIBLE,
                ..Default::default()
            },
        );
        map
    }
}

impl CreateInstanceWithId for Element {
    type Instance = Atom;

    fn create_instance(&self, id: Self::Id) -> Self::Instance {
        Atom {
            color: self.color,
            join_face: self.join_face,
            element: id,
        }
    }
}

impl Resource for IdMap<Element> {}

impl Element {
    pub const VOID_ID: <Self as MappedToId>::Id = 0;
    pub const AIR_ID: <Self as MappedToId>::Id = 0;
}

impl IdMap<Element> {
    pub fn air(&self) -> Atom {
        self.instance_of(Element::AIR_ID).unwrap()
    }
}
