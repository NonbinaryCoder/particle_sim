use bevy::{
    prelude::*,
    render::render_resource::encase::vector::{AsMutVectorParts, AsRefVectorParts},
};
use bevy_egui::{egui, EguiContexts};

use crate::ui::{ArrayMutWidget, ArrayWidget};

use super::{LookPos, Player};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_inspector_system);
    }
}

fn player_inspector_system(
    mut contexts: EguiContexts,
    mut player_query: Query<(&mut Transform, &LookPos), With<Player>>,
) {
    let (mut transform, look_pos) = player_query.single_mut();
    egui::Window::new("Player Inspector")
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label("Position:");
            ui.add(ArrayMutWidget(transform.translation.as_mut_parts()));

            ui.label("Looking:");
            ui.add(ArrayWidget(round(transform.forward(), 4).as_ref_parts()));

            if let Some(look_pos) = &look_pos.0 {
                ui.label("Target:");
                ui.add(ArrayWidget(look_pos.grid_pos.as_ref_parts()));
            }
        });
}

fn round(val: Vec3, places: u32) -> Vec3 {
    let v = 10.0_f32.powi(places as i32);
    (val * v).round() / v
}
