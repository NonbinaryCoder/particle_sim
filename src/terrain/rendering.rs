use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

use super::{storage::DEFAULT_SIZE, Atom, ByOpacity};

pub mod mesh_gen;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            mesh_gen::MeshGenPlugin,
            MaterialPlugin::<TerrainMaterial>::default(),
        ))
        .add_systems(PreStartup, create_terrain_materials_system)
        .add_systems(Startup, create_floor_system);
    }
}

/// Materials used by atoms and the floor.
pub type TerrainMaterials = ByOpacity<Handle<TerrainMaterial>>;

impl Resource for TerrainMaterials {}

#[derive(Debug, AsBindGroup, TypeUuid, Clone, TypePath)]
#[uuid = "46c0094b-ce2b-4c35-ac23-49388d7428ab"]
pub struct TerrainMaterial {
    transparent: bool,
}

impl Material for TerrainMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/opaque.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/opaque.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }

    fn alpha_mode(&self) -> AlphaMode {
        match self.transparent {
            true => AlphaMode::Premultiplied,
            false => AlphaMode::Opaque,
        }
    }
}

/// Startup system that creates the materials used for atoms and the floor.
fn create_terrain_materials_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    commands.insert_resource(TerrainMaterials {
        opaque: materials.add(TerrainMaterial { transparent: false }),
        transparent: materials.add(TerrainMaterial { transparent: true }),
    })
}

/// Startup system that spawns in the floor for the world.
fn create_floor_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TerrainMaterials>,
) {
    let mesh = meshes.add(mesh_gen::cube());
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
    by_opacity: ByOpacity<ChunkDataByOpacity>,
}

/// Data associated with each render chunk that is duplicated for each opacity
/// class.
#[derive(Debug, Default, Clone)]
pub struct ChunkDataByOpacity {
    atoms: u16,
    mesh: Option<(Entity, Handle<Mesh>)>,
}

impl ChunkData {
    pub fn atom_changed(&mut self, old: &Atom, new: &Atom) {
        self.is_changed = true;
        macro_rules! update_count {
            ($( $count:ident ).+, $fn:expr) => {
                Self::update_count(&mut self.$( $count ).+, $fn, old, new);
            };
        }

        update_count!(by_opacity.opaque.atoms, Atom::is_opaque);
        update_count!(by_opacity.transparent.atoms, Atom::is_transparent);
    }

    fn update_count(count: &mut u16, mut f: impl FnMut(&Atom) -> bool, old: &Atom, new: &Atom) {
        *count = count.wrapping_add_signed(f(new) as i16 - f(old) as i16);
    }
}
