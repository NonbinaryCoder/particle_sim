//! Atom and floor rendering.

use bevy::prelude::*;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_systems(
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
            translation: Vec3::new(0.0, -1.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::new(16.0, 1.0, 16.0),
        },
        ..default()
    });
}
