//! Atom and floor rendering, and atom physics.

use std::{
    f32::consts::PI,
    ops::{Index, IndexMut, Not},
};

use bevy::prelude::*;

use crate::atom_physics::{
    element::{Element, ElementId},
    id::IdMap,
};

use self::{color::AtomColor, storage::Atoms};

pub mod change_detection;
pub mod color;
pub mod rendering;
pub mod storage;
pub mod thread;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((rendering::RenderingPlugin, thread::ThreadPlugin))
            .init_resource::<Atoms>();
    }
}

#[derive(Debug)]
pub struct AtomWorld {
    pub atoms: Atoms,
    pub elements: IdMap<Element>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Atom {
    pub color: AtomColor,
    pub join_face: JoinFace,
    pub element: ElementId,
}

impl Default for Atom {
    fn default() -> Self {
        Self::AIR
    }
}

impl Atom {
    pub const VOID: Self = Self {
        color: AtomColor::INVISIBLE,
        join_face: JoinFace::DEFAULT,
        element: Element::VOID_ID,
    };

    pub const AIR: Self = Self {
        color: AtomColor::INVISIBLE,
        join_face: JoinFace::DEFAULT,
        element: Element::AIR_ID,
    };

    pub const fn is_visible(&self) -> bool {
        self.color.a > 0
    }

    pub const fn is_opaque(&self) -> bool {
        self.color.a == u8::MAX
    }

    pub const fn is_transparent(&self) -> bool {
        self.color.a < u8::MAX && self.color.a > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinFace {
    Never,
    SameAlpha,
}

impl JoinFace {
    pub const DEFAULT: Self = Self::SameAlpha;
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

    pub fn rotation(self) -> Quat {
        match self {
            Direction::PosX => Quat::from_rotation_y(PI * 0.5),
            Direction::NegX => Quat::from_rotation_y(PI * 0.5),
            Direction::PosY => Quat::from_rotation_x(PI * 0.5),
            Direction::NegY => Quat::from_rotation_x(PI * 0.5),
            Direction::PosZ => Quat::IDENTITY,
            Direction::NegZ => Quat::IDENTITY,
        }
    }

    pub fn from_vec3(v: Vec3) -> Direction {
        let mag = v.abs();

        macro_rules! process {
            ($v:ident, $pos:ident | $neg:ident) => {
                match v.$v >= 0.0 {
                    true => Direction::$pos,
                    false => Direction::$neg,
                }
            };
        }

        #[allow(clippy::collapsible_else_if)]
        if mag.x > mag.y && mag.x > mag.z {
            process!(x, PosX | NegX)
        } else if mag.y > mag.x && mag.y > mag.z {
            process!(y, PosY | NegY)
        } else {
            process!(z, PosZ | NegZ)
        }
    }
}

impl Not for Direction {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Direction::PosX => Direction::NegX,
            Direction::NegX => Direction::PosX,
            Direction::PosY => Direction::NegY,
            Direction::NegY => Direction::PosY,
            Direction::PosZ => Direction::NegZ,
            Direction::NegZ => Direction::PosZ,
        }
    }
}

fn world_to_grid_pos(pos: Vec3) -> IVec3 {
    pos.round().as_ivec3()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opacity {
    Opaque,
    Transparent,
}

impl Opacity {
    pub const VARIANTS: [Self; 2] = [Opacity::Opaque, Opacity::Transparent];
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ByOpacity<T> {
    pub opaque: T,
    pub transparent: T,
}

impl<T> Index<Opacity> for ByOpacity<T> {
    type Output = T;

    fn index(&self, index: Opacity) -> &Self::Output {
        match index {
            Opacity::Opaque => &self.opaque,
            Opacity::Transparent => &self.transparent,
        }
    }
}

impl<T> IndexMut<Opacity> for ByOpacity<T> {
    fn index_mut(&mut self, index: Opacity) -> &mut Self::Output {
        match index {
            Opacity::Opaque => &mut self.opaque,
            Opacity::Transparent => &mut self.transparent,
        }
    }
}
