use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContexts};

use super::ChunkMesh;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin)
            .add_systems(PostUpdate, mesh_inspector_system);
    }
}

fn mesh_inspector_system(
    mut contexts: EguiContexts,
    mut wireframe_config: ResMut<WireframeConfig>,
    mesh_query: Query<(&ChunkMesh, &Handle<Mesh>, &Transform)>,
    meshes: Res<Assets<Mesh>>,
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
        });
}
