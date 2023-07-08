use std::ops::{Deref, Index};

use bevy::prelude::*;

use crate::terrain::rendering::CHUNK_SIZE;

use self::array3d::Array3d;

use super::{rendering::ChunkData, Atom};

mod array3d;

type AtomsCurve = array3d::SimpleCurve;

type ChunksCurve = array3d::SimpleCurve;
type ChunksCurveIterMut<'a> = array3d::SimpleCurveIterMut<'a, ChunkData>;

pub const DEFAULT_SIZE: UVec3 = UVec3::new(128, 48, 256);

#[derive(Debug, Clone, Resource)]
pub struct Atoms {
    atoms: Array3d<Atom, AtomsCurve>,
    chunks: Array3d<ChunkData, ChunksCurve>,
}

impl Default for Atoms {
    fn default() -> Self {
        // Program might not work properly if world size is not a multiple of
        // CHUNK_SIZE, so panic here if that happens accidentally.
        assert_eq!(
            DEFAULT_SIZE,
            DEFAULT_SIZE / CHUNK_SIZE as u32 * CHUNK_SIZE as u32
        );
        Self {
            atoms: Array3d::new(DEFAULT_SIZE),
            chunks: Array3d::new(DEFAULT_SIZE / CHUNK_SIZE as u32),
        }
    }
}

impl Index<UVec3> for Atoms {
    type Output = Atom;

    fn index(&self, index: UVec3) -> &Self::Output {
        self.atoms.get(index).unwrap_or(&Atom::VOID)
    }
}

impl Index<IVec3> for Atoms {
    type Output = Atom;

    fn index(&self, index: IVec3) -> &Self::Output {
        // Negative fields will still be outside of range after bitcast.
        &self[index.as_uvec3()]
    }
}

impl Atoms {
    /// Sets the atom at the specified position.
    pub fn set(&mut self, pos: UVec3, atom: Atom) {
        let (old_atom, chunk) = self.atom_mut(pos);
        if *old_atom != atom {
            chunk.atom_changed(old_atom, &atom);
            *old_atom = atom;
        }
    }

    /// Returns a mutable reference to an atom and the data for the chunk it is
    /// in.  Note that changing an atom without updating chunk data may result
    /// in incorrect behavior.
    fn atom_mut(&mut self, pos: UVec3) -> (&mut Atom, &mut ChunkData) {
        (
            &mut self.atoms[pos],
            &mut self.chunks[pos / CHUNK_SIZE as u32],
        )
    }

    pub fn contains_pos(&self, pos: UVec3) -> bool {
        pos.x < self.size().x && pos.y < self.size().y && pos.z < self.size().z
    }

    pub fn chunks(&mut self) -> Chunks {
        Chunks {
            chunk_iter: self.chunks.iter_mut_labeled(),
            atoms: &self.atoms,
        }
    }

    pub const fn size(&self) -> UVec3 {
        self.atoms.size()
    }
}

pub struct Chunks<'a> {
    chunk_iter: ChunksCurveIterMut<'a>,
    atoms: &'a Array3d<Atom, AtomsCurve>,
}

impl<'a> Iterator for Chunks<'a> {
    type Item = (UVec3, Chunk<'a>, &'a mut ChunkData);

    fn next(&mut self) -> Option<Self::Item> {
        self.chunk_iter.next().map(|(chunk_data, pos)| {
            let pos = pos * CHUNK_SIZE as u32;
            (
                pos,
                Chunk {
                    pos,
                    offset: UVec3::new(u32::MAX, 0, 0),
                    atoms: self.atoms,
                },
                chunk_data,
            )
        })
    }
}

pub struct Chunk<'a> {
    pos: UVec3,
    offset: UVec3,
    atoms: &'a Array3d<Atom, AtomsCurve>,
}

impl<'a> Iterator for Chunk<'a> {
    type Item = AtomRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.offset.x = self.offset.x.wrapping_add(1);
        let try_x = self.offset.x % CHUNK_SIZE as u32;
        if self.offset.x != try_x {
            self.offset.x = try_x;
            self.offset.z += 1;
            let try_z = self.offset.z % CHUNK_SIZE as u32;
            if self.offset.z != try_z {
                self.offset.z = try_z;
                self.offset.y += 1;
            }
        }
        (self.offset.y < CHUNK_SIZE as u32).then_some(AtomRef {
            pos: self.pos + self.offset,
            atoms: self.atoms,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AtomRef<'a> {
    pos: UVec3,
    atoms: &'a Array3d<Atom, AtomsCurve>,
}

impl<'a> Deref for AtomRef<'a> {
    type Target = Atom;

    fn deref(&self) -> &Self::Target {
        &self.atoms[self.pos]
    }
}

impl<'a> AtomRef<'a> {
    pub fn pos(&self) -> UVec3 {
        self.pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_iter_correct_length() {
        let mut count = 0;
        let mut world = Atoms::default();
        let (_, chunk, _) = world.chunks().nth(3).unwrap();
        for _ in chunk {
            count += 1;
        }
        assert_eq!(count, CHUNK_SIZE.pow(3));
    }
}
