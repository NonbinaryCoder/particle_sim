//! Atom and floor rendering, and atom physics.

use bevy::prelude::*;

use self::{color::AtomColor, storage::Atoms};

mod color;
mod editing;
mod rendering;
pub mod storage;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((editing::EditingPlugin, rendering::RenderingPlugin))
            .init_resource::<Atoms>();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Atom {
    pub color: AtomColor,
}

impl Default for Atom {
    fn default() -> Self {
        Self::AIR
    }
}

impl Atom {
    pub const VOID: Self = Self {
        color: AtomColor::INVISIBLE,
    };

    pub const AIR: Self = Self {
        color: AtomColor::INVISIBLE,
    };

    pub fn is_visible(&self) -> bool {
        self.color.a > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

impl Direction {
    pub const DIRECTIONS: [Self; 6] = [
        Direction::PosX,
        Direction::NegX,
        Direction::PosY,
        Direction::NegY,
        Direction::PosZ,
        Direction::NegZ,
    ];

    pub const fn shading(self) -> f32 {
        match self {
            Direction::PosX | Direction::NegX => 0.8,
            Direction::PosY | Direction::NegY => 0.9834,
            Direction::PosZ | Direction::NegZ => 0.88,
        }
    }

    pub const fn normal(self) -> Vec3 {
        match self {
            Direction::PosX => Vec3::X,
            Direction::NegX => Vec3::NEG_X,
            Direction::PosY => Vec3::Y,
            Direction::NegY => Vec3::NEG_Y,
            Direction::PosZ => Vec3::Z,
            Direction::NegZ => Vec3::NEG_Z,
        }
    }

    pub const fn normal_ivec(self) -> IVec3 {
        match self {
            Direction::PosX => IVec3::X,
            Direction::NegX => IVec3::NEG_X,
            Direction::PosY => IVec3::Y,
            Direction::NegY => IVec3::NEG_Y,
            Direction::PosZ => IVec3::Z,
            Direction::NegZ => IVec3::NEG_Z,
        }
    }

    pub const fn tangent(self) -> Vec3 {
        match self {
            Direction::PosX => Vec3::Z,
            Direction::NegX => Vec3::Y,
            Direction::PosY => Vec3::X,
            Direction::NegY => Vec3::Z,
            Direction::PosZ => Vec3::Y,
            Direction::NegZ => Vec3::X,
        }
    }

    pub const fn bitangent(self) -> Vec3 {
        match self {
            Direction::PosX => Vec3::Y,
            Direction::NegX => Vec3::Z,
            Direction::PosY => Vec3::Z,
            Direction::NegY => Vec3::X,
            Direction::PosZ => Vec3::X,
            Direction::NegZ => Vec3::Y,
        }
    }
}

fn world_to_grid_pos(pos: Vec3) -> IVec3 {
    pos.round().as_ivec3()
}
