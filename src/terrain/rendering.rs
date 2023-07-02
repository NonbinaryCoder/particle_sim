use bevy::prelude::*;

use super::{storage::DEFAULT_SIZE, Atom};

mod mesh_gen;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(mesh_gen::MeshGenPlugin).add_startup_systems(
            (
                create_terrain_materials_system,
                apply_system_buffers,
                create_floor_system,
            )
                .chain(),
        );
    }
}

/// Materials used by atoms and the floor.
#[derive(Debug, Resource)]
struct TerrainMaterials {
    opaque: Handle<StandardMaterial>,
}

impl FromWorld for TerrainMaterials {
    fn from_world(world: &mut World) -> Self {
        Self {
            opaque: world
                .resource_mut::<Assets<TerrainMaterial>>()
                .add(StandardMaterial::default()),
        }
    }
}

/// Material type used by atoms and the floor.
type TerrainMaterial = StandardMaterial;

/// Startup system that creates the materials used for atoms and the floor.
fn create_terrain_materials_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    commands.insert_resource(TerrainMaterials {
        opaque: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..Default::default()
        }),
    })
}

/// Startup system that spawns in the floor for the world.
fn create_floor_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TerrainMaterials>,
) {
    let mesh = meshes.add(shape::Cube::new(1.0).into());
    commands.spawn(MaterialMeshBundle {
        mesh,
        material: materials.opaque.clone(),
        transform: Transform {
            translation: Vec3::new(
                DEFAULT_SIZE.x as f32 * 0.5 - 0.5,
                -1.0,
                DEFAULT_SIZE.z as f32 * 0.5 - 0.5,
            ),
            rotation: Quat::IDENTITY,
            scale: Vec3::new(DEFAULT_SIZE.x as f32, 1.0, DEFAULT_SIZE.z as f32),
        },
        ..default()
    });
}

pub const CHUNK_SIZE: usize = 16;

/// Data associated with each render chunk.
#[derive(Debug, Default, Clone)]
pub struct ChunkData {
    is_changed: bool,
    visible_atom_count: u16,
    mesh: Option<(Entity, Handle<Mesh>)>,
}

impl ChunkData {
    pub fn atom_changed(&mut self, old: &Atom, new: &Atom) {
        self.is_changed = true;
        Self::update_count(&mut self.visible_atom_count, Atom::is_visible, old, new);
    }

    fn update_count(count: &mut u16, mut f: impl FnMut(&Atom) -> bool, old: &Atom, new: &Atom) {
        *count = count.wrapping_add_signed(f(new) as i16 - f(old) as i16);
    }
}
