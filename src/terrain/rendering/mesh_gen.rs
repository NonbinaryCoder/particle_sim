use std::iter;

use bevy::{
    prelude::*,
    render::{mesh::VertexAttributeValues, primitives::Aabb, render_resource::PrimitiveTopology},
};

use crate::terrain::{
    color::{AtomColor, UncompressedColor},
    storage::{AtomRef, Atoms, Chunk},
    Direction, JoinFace, Opacity,
};

use super::{ChunkData, ChunkDataByOpacity, TerrainMaterials};

mod inspector;

pub struct MeshGenPlugin;

impl Plugin for MeshGenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(inspector::InspectorPlugin)
            .add_systems(Update, generate_chunk_meshes_system);
    }
}

fn generate_chunk_meshes_system(
    mut commands: Commands,
    mut aabb_query: Query<&mut Aabb>,
    mut world: ResMut<Atoms>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TerrainMaterials>,
) {
    for (pos, chunk, chunk_data) in world.chunks() {
        if chunk_data.is_changed {
            chunk_data.is_changed = false;
            modify_chunk_meshes(
                &mut commands,
                &mut aabb_query,
                &mut meshes,
                &materials,
                chunk,
                chunk_data,
                pos,
            );
        }
    }
}

fn modify_chunk_meshes(
    commands: &mut Commands,
    aabb_query: &mut Query<&mut Aabb>,
    meshes: &mut Assets<Mesh>,
    materials: &TerrainMaterials,
    chunk: Chunk,
    data: &mut ChunkData,
    pos: UVec3,
) {
    for typ in Opacity::VARIANTS {
        modify_chunk_mesh(
            commands,
            aabb_query,
            meshes,
            materials,
            chunk.clone(),
            data,
            pos,
            typ,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn modify_chunk_mesh(
    commands: &mut Commands,
    aabb_query: &mut Query<&mut Aabb>,
    meshes: &mut Assets<Mesh>,
    materials: &TerrainMaterials,
    chunk: Chunk,
    data: &mut ChunkData,
    pos: UVec3,
    opacity: Opacity,
) {
    let data = &mut data.by_opacity[opacity];
    if data.atoms > 0 {
        let (entity, mesh) = get_entity_and_mesh(commands, meshes, materials, data, pos, opacity);
        let mut builder = MeshBuilder::extract(mesh);
        builder.clear();

        match opacity {
            Opacity::Opaque => generate_chunk_mesh_opaque(chunk, pos, data, builder),
            Opacity::Transparent => generate_chunk_mesh_transparent(chunk, pos, data, builder),
        }

        let aabb = mesh.compute_aabb().unwrap_or_default();
        if let Ok(mut mesh_aabb) = aabb_query.get_mut(entity) {
            *mesh_aabb = aabb;
        } else {
            commands.entity(entity).insert(aabb);
        }
    } else if let Some((entity, _)) = data.mesh.take() {
        commands.entity(entity).despawn();
        // No need to clean up mesh because it will be removed when its handle
        // is dropped.
    }
}

fn get_entity_and_mesh<'m>(
    commands: &mut Commands,
    meshes: &'m mut Assets<Mesh>,
    materials: &TerrainMaterials,
    data: &mut ChunkDataByOpacity,
    pos: UVec3,
    opacity: Opacity,
) -> (Entity, &'m mut Mesh) {
    let (entity, mesh) = data
        .mesh
        .get_or_insert_with(|| init_chunk_mesh(commands, meshes, materials, pos, opacity));
    (*entity, meshes.get_mut(mesh).unwrap())
}

fn generate_chunk_mesh_opaque(
    chunk: Chunk,
    pos: UVec3,
    data: &ChunkDataByOpacity,
    mut mesh: MeshBuilder,
) {
    let mut atoms_rendered = 0;
    for atom in chunk {
        if atom.is_opaque() {
            generate_atom_mesh_join_same_alpha(atom, pos, &mut mesh);
            atoms_rendered += 1;
            if atoms_rendered == data.atoms {
                break;
            }
        }
    }
}

fn generate_chunk_mesh_transparent(
    chunk: Chunk,
    pos: UVec3,
    data: &ChunkDataByOpacity,
    mut mesh: MeshBuilder,
) {
    let mut atoms_rendered = 0;
    for atom in chunk {
        if atom.is_transparent() {
            atoms_rendered += 1;
            match atom.join_face {
                JoinFace::Never => generate_atom_mesh_never_join(atom, pos, &mut mesh),
                JoinFace::SameAlpha => generate_atom_mesh_join_same_alpha(atom, pos, &mut mesh),
            }
            if atoms_rendered == data.atoms {
                break;
            }
        }
    }
}

fn generate_atom_mesh_never_join(atom: AtomRef, pos: UVec3, mesh: &mut MeshBuilder) {
    let color = atom.color.decompress();
    for direction in Direction::DIRECTIONS {
        mesh.add_face(atom.pos() - pos, color, direction);
    }
}

fn generate_atom_mesh_join_same_alpha(atom: AtomRef, pos: UVec3, mesh: &mut MeshBuilder) {
    let color = atom.color.decompress();
    for direction in Direction::DIRECTIONS {
        if atom.in_direction(direction).color.a != atom.color.a {
            mesh.add_face(atom.pos() - pos, color, direction);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct ChunkMesh {
    pos: UVec3,
}

fn init_chunk_mesh(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &TerrainMaterials,
    pos: UVec3,
    opacity: Opacity,
) -> (Entity, Handle<Mesh>) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new());
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new());
    let mesh = meshes.add(mesh);

    let entity = commands
        .spawn((
            MaterialMeshBundle {
                mesh: mesh.clone(),
                material: materials[opacity].clone(),
                transform: Transform::from_translation(pos.as_vec3()),
                ..default()
            },
            ChunkMesh { pos },
            Aabb::default(),
        ))
        .id();

    (entity, mesh)
}

#[derive(Debug)]
struct MeshBuilder<'a> {
    position: &'a mut Vec<[f32; 3]>,
    color: &'a mut Vec<[f32; 4]>,
}

impl<'a> MeshBuilder<'a> {
    fn extract(mesh: &mut Mesh) -> MeshBuilder<'_> {
        let mut position = None;
        let mut color = None;

        for (id, values) in mesh.attributes_mut() {
            if id == Mesh::ATTRIBUTE_POSITION.id {
                let VertexAttributeValues::Float32x3(p) = values else { panic!(
                   "position should be `Float32x3` but is `{}``",
                    values.enum_variant_name()
                ) };
                position = Some(p);
            } else if id == Mesh::ATTRIBUTE_COLOR.id {
                let VertexAttributeValues::Float32x4(p) = values else { panic!(
                    "color should be `Float32x4` but is `{}``",
                    values.enum_variant_name()
                ) };
                color = Some(p);
            } else {
                panic!("Unexpected mesh attribute: {id:?}")
            }
        }

        let position = position.expect("Terrain mesh missing position");
        let color = color.expect("Terrain mesh missing color");

        MeshBuilder { position, color }
    }

    fn clear(&mut self) {
        self.position.clear();
        self.color.clear();
    }

    fn reserve(&mut self, faces: usize) {
        // 6 vertices per face.
        let num_vertices = faces * 6;
        self.position.reserve(num_vertices);
        self.color.reserve(num_vertices);
    }

    fn add_face(&mut self, pos: UVec3, color: UncompressedColor, direction: Direction) {
        let pos = pos.as_vec3();

        let normal = direction.normal();
        let center = pos + normal * 0.5;

        let tangent = direction.tangent() * 0.5;
        let bitangent = direction.bitangent() * 0.5;

        let corners = [
            center - tangent - bitangent,
            center + tangent - bitangent,
            center - tangent + bitangent,
            center + tangent + bitangent,
        ];

        self.add_quad(corners, color.to_mesh_color(direction.shading()));
    }

    /// 0-1
    /// |/|
    /// 2-3
    fn add_quad(&mut self, corners: [Vec3; 4], color: [f32; 4]) {
        let points = corners.map(|p| p.to_array());
        self.position.extend([
            points[2], points[1], points[0], points[2], points[3], points[1],
        ]);
        self.color.extend(iter::repeat(color).take(6));
    }
}

pub fn cube() -> Mesh {
    let mut position = Vec::new();
    let mut color = Vec::new();
    let mut builder = MeshBuilder {
        position: &mut position,
        color: &mut color,
    };

    builder.reserve(6);
    for direction in Direction::DIRECTIONS {
        builder.add_face(UVec3::ZERO, AtomColor::WHITE.decompress(), direction);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, position);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, color);

    mesh
}
