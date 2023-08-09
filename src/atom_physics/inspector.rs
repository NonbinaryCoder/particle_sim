use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::player::SelectedElement;

use super::{
    element::Element,
    id::IdMap,
    io::{AvalibleSets, LoadSet},
};

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
    elements: Res<IdMap<Element>>,
    mut selected_element: ResMut<SelectedElement>,
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
            for (id, name, _) in elements.iter() {
                if ui
                    .selectable_label(selected_element.0 == id, name)
                    .clicked()
                {
                    selected_element.0 = id
                }
            }

            if let Some(element) = elements.get(selected_element.0) {
                ui.label(format!("{:#?}", element));
            }

            ui.separator();
            if ui.button("Reload").clicked() {
                load_set.send(LoadSet {
                    name: avalible_sets.get(*selected_set).map(|s| s.name.clone()),
                });
            }
        });
}
