use std::{
    iter,
    ops::{Index, IndexMut},
};

use bevy::prelude::*;

/// A 3D array with options to specify how data is laid out in memory.
#[derive(Debug, Clone)]
pub struct Array3d<T, C> {
    data: Box<[T]>,
    size: UVec3,
    curve: C,
}

impl<T> Array3d<T, SimpleCurve> {
    /// Uses the default mapping.
    pub fn new(size: UVec3) -> Array3d<T, SimpleCurve>
    where
        T: Default,
    {
        Array3d::new_with_curve(size, SimpleCurve)
    }

    pub const fn size(&self) -> UVec3 {
        self.size
    }

    /// Returns a mutable reference to the element at the index, or `None` if
    /// the index is not within the bounds of this.
    pub fn get(&self, index: UVec3) -> Option<&T> {
        (index.x < self.size.x && index.y < self.size.y && index.z < self.size.z)
            .then(|| &self[index])
    }
}

impl<T, C: SpaceFillingCurve> Array3d<T, C> {
    pub fn new_with_curve(size: UVec3, curve: C) -> Self
    where
        T: Default,
    {
        let area = curve.data_length(size);

        let mut data = Vec::with_capacity(area);
        data.extend(iter::repeat_with(T::default).take(area));
        let data = data.into_boxed_slice();

        Self { data, size, curve }
    }
}

impl<T, C: IterableCurve> Array3d<T, C> {
    pub fn iter_mut_labeled(&mut self) -> C::IterMut<'_, T> {
        self.curve.iter_mut_labeled(&mut self.data, self.size)
    }
}

impl<T, C: SpaceFillingCurve> Index<UVec3> for Array3d<T, C> {
    type Output = T;

    fn index(&self, index: UVec3) -> &Self::Output {
        let index = self.curve.index_of(self.size, index);
        &self.data[index]
    }
}

impl<T, C: SpaceFillingCurve> IndexMut<UVec3> for Array3d<T, C> {
    fn index_mut(&mut self, index: UVec3) -> &mut Self::Output {
        let index = self.curve.index_of(self.size, index);
        &mut self.data[index]
    }
}

/// A mapping from points in 3D space to points in 1D space.
pub trait SpaceFillingCurve {
    fn data_length(&self, size: UVec3) -> usize {
        size.x as usize * size.y as usize * size.z as usize
    }

    fn index_of(&self, size: UVec3, pos: UVec3) -> usize;
}

/// A space filling curve that provides a way to iterate over an [`Array3d`] and
/// label each element with it's position.
pub trait IterableCurve: SpaceFillingCurve + Sized {
    type IterMut<'a, T>: Iterator<Item = (&'a mut T, UVec3)>
    where
        Self: 'a,
        T: 'a;

    fn iter_mut_labeled<'a, T>(&'a self, data: &'a mut [T], size: UVec3) -> Self::IterMut<'a, T>;
}

/// Simpleist mapping from 3D to 1D space.
///
/// Very fast, but poor cache efficiency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimpleCurve;

impl SpaceFillingCurve for SimpleCurve {
    fn index_of(&self, size: UVec3, pos: UVec3) -> usize {
        (pos.x + pos.z * size.x + pos.y * size.x * size.z) as usize
    }
}

impl IterableCurve for SimpleCurve {
    type IterMut<'a, T> = SimpleCurveIterMut<'a, T> where T: 'a;

    fn iter_mut_labeled<'a, T>(&'a self, data: &'a mut [T], size: UVec3) -> Self::IterMut<'a, T> {
        SimpleCurveIterMut {
            iter: data.iter_mut(),
            size,
            index: UVec3::new(u32::MAX, 0, 0),
        }
    }
}

pub struct SimpleCurveIterMut<'a, T> {
    iter: std::slice::IterMut<'a, T>,
    size: UVec3,
    index: UVec3,
}

impl<'a, T> Iterator for SimpleCurveIterMut<'a, T> {
    type Item = (&'a mut T, UVec3);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| {
            self.index.x = self.index.x.wrapping_add(1);
            if self.index.x == self.size.x {
                self.index.x = 0;
                self.index.z += 1;
                if self.index.z == self.size.z {
                    self.index.z = 0;
                    self.index.y += 1;
                }
            }

            (item, self.index)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_curve(size: UVec3, curve: impl SpaceFillingCurve) {
        let mut world: Array3d<Option<UVec3>, _> = Array3d::new_with_curve(size, curve);
        for x in 0..size.x {
            for y in 0..size.y {
                for z in 0..size.z {
                    let pos = UVec3 { x, y, z };
                    let index = world.curve.index_of(world.size, pos);
                    let arr_len = world.data.len();
                    if index >= arr_len {
                        panic!("Index for point {pos} not in array (size: {size}, array length: {arr_len}, index: {index})");
                    }
                    if let Some(last_pos) = world.data[index] {
                        panic!("Point {pos} generates index {index}, but that index is already taken by {last_pos} (size: {size})");
                    }
                    world.data[index] = Some(pos);
                }
            }
        }
    }

    #[test]
    fn simple_curve() {
        test_curve(UVec3::new(8, 8, 8), SimpleCurve);
        test_curve(UVec3::new(256, 16, 8), SimpleCurve);
        test_curve(UVec3::new(16, 256, 8), SimpleCurve);
        test_curve(UVec3::new(8, 16, 256), SimpleCurve);
        test_curve(UVec3::new(256, 256, 256), SimpleCurve);
    }
}
