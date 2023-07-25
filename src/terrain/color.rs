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

    pub const WHITE: Self = Self::from_u32(0xffffffff);

    /// 0xrrggbbaa
    pub const fn from_u32(val: u32) -> Self {
        let [r, g, b, a] = val.to_be_bytes();
        Self { r, g, b, a }
    }

    pub const fn from_parts(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn decompress(self) -> UncompressedColor {
        let [r, g, b, a] = (Color::rgba_u8(self.r, self.g, self.b, self.a)).as_rgba_f32();
        UncompressedColor([r * a, g * a, b * a, a])
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UncompressedColor([f32; 4]);

impl UncompressedColor {
    pub fn to_mesh_color(self, shading: f32) -> [f32; 4] {
        let [r, g, b, a] = self.0;
        [r * shading, g * shading, b * shading, a]
    }
}
