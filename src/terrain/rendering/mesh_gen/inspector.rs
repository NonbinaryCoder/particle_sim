use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContexts};

use crate::{
    player::Player,
    terrain::{self, rendering::CHUNK_SIZE, storage::Atoms},
};

use super::ChunkMesh;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin)
            .add_systems(Update, mesh_inspector_system);
    }
}

#[allow(clippy::too_many_arguments)]
fn mesh_inspector_system(
    mut gizmos: Gizmos,
    mut contexts: EguiContexts,
    mut wireframe_config: ResMut<WireframeConfig>,
    mesh_query: Query<(&ChunkMesh, &Handle<Mesh>, &Transform)>,
    meshes: Res<Assets<Mesh>>,
    player_query: Query<&Transform, With<Player>>,
    mut show_chunk_borders: Local<bool>,
    world: Res<Atoms>,
) {
    egui::Window::new("Mesh Inspector")
        .default_width(200.0)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            for (pos, mesh, transform) in &mesh_query {
                let mesh = meshes.get(mesh).unwrap();
                ui.label(format!("{}", pos.pos));
                ui.label(format!(
                    "- tricount: {}",
                    mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().len() / 3
                ));
                ui.label(format!("- position: {}", transform.translation));
            }

            ui.separator();

            let mut wireframe = wireframe_config.global;
            ui.checkbox(&mut wireframe, "Wireframe");
            if wireframe != wireframe_config.global {
                wireframe_config.global = wireframe;
            }

            ui.checkbox(&mut show_chunk_borders, "Show chunk borders");
            if *show_chunk_borders {
                let player_pos = terrain::world_to_grid_pos(player_query.single().translation);
                if world.contains_atom(player_pos) {
                    let chunk = player_pos / CHUNK_SIZE as i32;
                    let chunk_center =
                        chunk.as_vec3() * CHUNK_SIZE as f32 + CHUNK_SIZE as f32 * 0.5 - 0.5;

                    let chunk_extent = CHUNK_SIZE as f32 / 2.0;

                    gizmos.cuboid(
                        Transform::from_translation(chunk_center)
                            .with_scale(Vec3::splat(CHUNK_SIZE as f32)),
                        Color::RED,
                    );

                    for [up, a, b] in [
                        [Vec3::Z, Vec3::X, Vec3::Y],
                        [Vec3::Y, Vec3::X, Vec3::Z],
                        [Vec3::X, Vec3::Y, Vec3::Z],
                    ] {
                        let mut bottom = chunk_center - up * chunk_extent;
                        let up = up * 2.0;
                        let a = a * chunk_extent;
                        let b = b * chunk_extent;
                        for _ in 1..CHUNK_SIZE / 2 {
                            bottom += up;
                            gizmos.linestrip(
                                [
                                    bottom + a + b,
                                    bottom + a - b,
                                    bottom - a - b,
                                    bottom - a + b,
                                    bottom + a + b,
                                ],
                                Color::YELLOW,
                            );
                        }
                    }
                }
            }
        });
}
