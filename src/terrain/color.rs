use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtomColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl AtomColor {
    pub const INVISIBLE: Self = Self::from_u32(0);

    /// 0xrrggbbaa
    pub const fn from_u32(val: u32) -> Self {
        let [r, g, b, a] = val.to_be_bytes();
        Self { r, g, b, a }
    }

    pub fn to_mesh_color(self) -> [f32; 4] {
        Color::rgba_u8(self.r, self.g, self.b, self.a).as_rgba_f32()
    }
}
