use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::ui::Vec3Widget;

use super::Player;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_inspector_system);
    }
}

fn player_inspector_system(
    mut contexts: EguiContexts,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let mut transform = player_query.single_mut();
    egui::Window::new("Player Inspector")
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label("Position:");
            ui.add(Vec3Widget(&mut transform.translation));
            ui.label("Looking:");
            ui.add(Vec3Widget(&mut transform.forward()));
        });
}
