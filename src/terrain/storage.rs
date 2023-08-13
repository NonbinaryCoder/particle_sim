use std::{
    mem,
    ops::{Deref, Index},
};

use bevy::prelude::*;

use crate::{
    physics::colliders::{Collides, Rect3d},
    terrain::rendering::CHUNK_SIZE,
};

use self::array3d::{Array3d, GridPos};

use super::{change_detection::DetectChanges, rendering::ChunkData, Atom, Direction};

mod array3d;

type AtomsCurve = array3d::SimpleCurve;

type ChunksCurve = array3d::SimpleCurve;
type ChunksCurveIterMut<'a> = array3d::SimpleCurveIterMut<'a, ChunkData>;

pub const DEFAULT_SIZE: UVec3 = UVec3::new(128, 48, 256);

#[derive(Debug, Clone)]
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

impl<T: GridPos> Index<T> for Atoms {
    type Output = Atom;

    fn index(&self, index: T) -> &Self::Output {
        // Negative fields will still be outside of range after bitcast.
        &self.atoms[index.to_uvec3()]
    }
}

impl Atoms {
    /// Sets the atom at the specified position.
    pub fn set(&mut self, pos: UVec3, atom: Atom) {
        let (old_atom, chunk, chunk_pos) = self.atom_mut(pos);
        if *old_atom != atom {
            chunk.atom_changed(old_atom, &atom);
            *old_atom = atom;
        }

        let pos = pos % CHUNK_SIZE as u32;
        macro_rules! update_adjacent {
            ($f:ident, $dir:ident) => {
                if pos.$f == 0 {
                    if chunk_pos.$f > 0 {
                        self.chunks[chunk_pos - UVec3::$dir].mark_changed();
                    }
                } else if pos.$f == CHUNK_SIZE as u32 - 1
                    && (chunk_pos.$f + 1) * 16 < self.size().$f
                {
                    self.chunks[chunk_pos + UVec3::$dir].mark_changed();
                }
            };
        }

        update_adjacent!(x, X);
        update_adjacent!(y, Y);
        update_adjacent!(z, Z);
    }

    /// Returns a mutable reference to an atom and the data for the chunk it is
    /// in.  Note that changing an atom without updating chunk data may result
    /// in incorrect behavior.
    fn atom_mut(&mut self, pos: UVec3) -> (&mut Atom, &mut ChunkData, UVec3) {
        let chunk_pos = pos / CHUNK_SIZE as u32;
        (&mut self.atoms[pos], &mut self.chunks[chunk_pos], chunk_pos)
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        let point = point - Vec3::splat(-0.5);
        point.cmpgt(Vec3::ZERO).all() && point.cmplt(self.size().as_vec3()).all()
    }

    pub fn contains_atom(&self, pos: impl GridPos) -> bool {
        let pos = pos.to_uvec3();
        pos.x < self.size().x && pos.y < self.size().y && pos.z < self.size().z
    }

    pub fn chunks(&mut self) -> Chunks {
        Chunks {
            chunk_iter: self.chunks.iter_mut_labeled(),
            atoms: &self.atoms,
        }
    }

    /// Grants mutable access to every atom sequentially, in a way that makes
    /// modification faster than using [`Self::set`] on each atom.
    pub fn modify_all(&mut self, mut f: impl FnMut(DetectChanges<Atom>)) {
        for (chunk_data, chunk_pos) in self.chunks.iter_mut_labeled() {
            let offset = chunk_pos * CHUNK_SIZE as u32;
            chunk_data.__reset_counts();
            for x in 0..CHUNK_SIZE as u32 {
                for y in 0..CHUNK_SIZE as u32 {
                    for z in 0..CHUNK_SIZE as u32 {
                        let atom = &mut self.atoms[offset + UVec3 { x, y, z }];
                        let mut changed = false;
                        f(DetectChanges::new(atom, &mut changed));

                        if changed {
                            chunk_data.mark_changed();
                        }
                        chunk_data.__add_atom(atom);
                    }
                }
            }
        }
    }

    pub const fn size(&self) -> UVec3 {
        self.atoms.size()
    }

    pub fn raycast(
        &self,
        ray: Ray,
        max_dist: f32,
        mut hit: impl FnMut(&Atom) -> bool,
    ) -> Option<RaycastHit> {
        let (dist, ray) = if self.contains_point(ray.origin) {
            let grid_pos = super::world_to_grid_pos(ray.origin);
            if hit(&self[grid_pos]) {
                return Some(RaycastHit {
                    grid_pos,
                    side: !Direction::from_vec3(ray.direction),
                    dist: 0.0,
                    is_wall: false,
                });
            }
            (0.0, ray)
        } else {
            let (dist, side) = self.raycast_walls(ray, max_dist)?;
            let grid_pos = super::world_to_grid_pos(ray.get_point(dist));
            if hit(&self[grid_pos]) {
                return Some(RaycastHit {
                    grid_pos,
                    side,
                    dist,
                    is_wall: false,
                });
            }
            (
                dist,
                Ray {
                    origin: ray.get_point(dist),
                    ..ray
                },
            )
        };

        self.raycast_terrain(ray, max_dist - dist, hit)
            .map(|hit| RaycastHit {
                dist: dist + hit.dist,
                ..hit
            })
    }

    fn raycast_walls(&self, ray: Ray, max_dist: f32) -> Option<(f32, Direction)> {
        let size = self.size().as_vec3();

        macro_rules! process {
            ($face:expr, $normal:expr, $side:ident) => {
                let face = $face;
                let normal = $normal;
                if ray.collides(&face) {
                    let dist = ray.intersect_plane(face.origin, normal).unwrap();
                    return Some((dist + 1.0 / 16.0, Direction::$side))
                        .filter(|&(dist, _)| dist <= max_dist);
                }
            };
        }
        fn flip(mut rect: Rect3d, movement: Vec3) -> Rect3d {
            mem::swap(&mut rect.extents_a, &mut rect.extents_b);
            rect.origin += movement;
            rect
        }

        /* Ceiling */
        let extents_a = Vec3::Z * size.z;
        let extents_b = Vec3::X * size.x;
        let ceiling = Rect3d {
            origin: (extents_a + extents_b) * 0.5 - Vec3::splat(0.5) + Vec3::Y * size.y,
            extents_a,
            extents_b,
        };

        process!(ceiling, Vec3::Y, NegY);

        /* x */
        let extents_a = Vec3::Z * size.z;
        let extents_b = Vec3::Y * size.y;
        let wall_x = Rect3d {
            origin: (extents_a + extents_b) * 0.5 - Vec3::splat(0.5),
            extents_a,
            extents_b,
        };
        process!(wall_x, Vec3::X, NegX);
        process!(flip(wall_x, Vec3::X * size.x), -Vec3::X, PosX);

        /* z */
        let extents_a = Vec3::Y * size.y;
        let extents_b = Vec3::X * size.x;
        let wall_z = Rect3d {
            origin: (extents_a + extents_b) * 0.5 - Vec3::splat(0.5),
            extents_a,
            extents_b,
        };
        process!(wall_z, Vec3::Z, NegZ);
        process!(flip(wall_z, Vec3::Z * size.z), -Vec3::Z, PosZ);

        None
    }

    fn raycast_terrain(
        &self,
        ray: Ray,
        max_dist: f32,
        mut hit: impl FnMut(&Atom) -> bool,
    ) -> Option<RaycastHit> {
        // From "A Fast Voxel Traversal Algorithm for Ray Tracing" by John
        // Amanatides and Andrew Woo, 1987.
        // [http://www.cse.yorku.ca/~amana/research/grid.pdf]
        // [http://citeseer.ist.psu.edu/viewdoc/summary?doi=10.1.1.42.3443]

        debug_assert!(ray.direction.is_normalized());

        /* Initialization */
        let mut atom = super::world_to_grid_pos(ray.origin);
        let step = ray.direction.signum();
        let atom_f32 = atom.as_vec3();
        let next_voxel_boundary = atom_f32 + step * 0.5;
        let next_voxel_boundary = next_voxel_boundary
            + Vec3::select(next_voxel_boundary.cmpeq(ray.origin), step, Vec3::ZERO);
        let mut t_max = Vec3 {
            x: ray
                .intersect_plane(next_voxel_boundary, Vec3::X * step)
                .unwrap_or(f32::INFINITY),
            y: ray
                .intersect_plane(next_voxel_boundary, Vec3::Y * step)
                .unwrap_or(f32::INFINITY),
            z: ray
                .intersect_plane(next_voxel_boundary, Vec3::Z * step)
                .unwrap_or(f32::INFINITY),
        };
        let ray_dir = Ray {
            origin: Vec3::ZERO,
            ..ray
        };
        let t_delta = Vec3 {
            x: ray_dir
                .intersect_plane(Vec3::X * step, Vec3::X * step)
                .unwrap_or(f32::INFINITY),
            y: ray_dir
                .intersect_plane(Vec3::Y * step, Vec3::Y * step)
                .unwrap_or(f32::INFINITY),
            z: ray_dir
                .intersect_plane(Vec3::Z * step, Vec3::Z * step)
                .unwrap_or(f32::INFINITY),
        };
        let step = step.as_ivec3();

        /* Incremental */
        macro_rules! process {
            ($v:ident, $pos:ident | $neg:ident) => {
                atom.$v += step.$v;
                let dist = t_max.min_element();
                t_max.$v += t_delta.$v;
                if !self.contains_atom(atom.as_uvec3()) {
                    return Some(RaycastHit {
                        grid_pos: atom,
                        side: if step.$v == 1 {
                            Direction::$pos
                        } else {
                            Direction::$neg
                        },
                        dist,
                        is_wall: true,
                    });
                }
                if hit(&self[atom]) {
                    return Some(RaycastHit {
                        grid_pos: atom,
                        side: if step.$v == 1 {
                            Direction::$pos
                        } else {
                            Direction::$neg
                        },
                        dist,
                        is_wall: false,
                    });
                }
            };
        }

        // Limit iterations to not get stuck in loop.
        #[allow(clippy::collapsible_else_if)]
        while t_max.min_element() <= max_dist {
            if t_max.x < t_max.y {
                if t_max.x < t_max.z {
                    process!(x, NegX | PosX);
                } else {
                    process!(z, NegZ | PosZ);
                }
            } else {
                if t_max.y < t_max.z {
                    process!(y, NegY | PosY);
                } else {
                    process!(z, NegZ | PosZ);
                }
            }
        }

        None
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

#[derive(Debug, Clone)]
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

    pub fn in_direction(&self, direction: Direction) -> &Atom {
        self.atoms
            .get_or(self.pos.as_ivec3() + direction.normal_ivec(), &Atom::VOID)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    pub grid_pos: IVec3,
    pub side: Direction,
    pub dist: f32,
    pub is_wall: bool,
}

#[cfg(test)]
mod tests {
    use crate::terrain::{color::AtomColor, JoinFace};

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

    #[test]
    fn all_atoms_placeable() {
        let mut world = Atoms::default();
        for x in 0..world.size().x {
            for y in 0..world.size().y {
                for z in 0..world.size().z {
                    world.set(
                        UVec3 { x, y, z },
                        Atom {
                            color: AtomColor::from_u32(0xff0000ff),
                            join_face: JoinFace::SameAlpha,
                            element: 2,
                        },
                    )
                }
            }
        }
    }
}
