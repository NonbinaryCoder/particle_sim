//! Atom and floor rendering, and atom physics.

use bevy::prelude::*;

use self::{color::AtomColor, storage::Atoms};

mod color;
mod editing;
mod rendering;
mod storage;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(editing::EditingPlugin)
            .add_plugin(rendering::RenderingPlugin)
            .init_resource::<Atoms>();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Atom {
    pub color: AtomColor,
}

impl Default for Atom {
    fn default() -> Self {
        Self {
            color: AtomColor::INVISIBLE,
        }
    }
}

impl Atom {
    pub const VOID: Self = Self {
        color: AtomColor::INVISIBLE,
    };

    pub fn is_visible(&self) -> bool {
        self.color.a > 0
    }
}
