use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::io::{AvalibleSets, LoadSet};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, set_inspector_system);
    }
}

pub fn set_inspector_system(
    mut contexts: EguiContexts,
    mut selected_set: Local<usize>,
    avalible_sets: Res<AvalibleSets>,
    mut load_set: EventWriter<LoadSet>,
) {
    egui::Window::new("Set Inspector")
        .default_width(200.0)
        .show(contexts.ctx_mut(), |ui| {
            egui::ComboBox::new(0x_e179335e0c420a1c_u64, "").show_index(
                ui,
                &mut selected_set,
                avalible_sets.len(),
                |i| &avalible_sets[i].name,
            );
            ui.separator();
            if ui.button("Reload").clicked() {
                load_set.send(LoadSet { name: None });
            }
        });
}
