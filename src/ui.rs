use bevy::{prelude::*, window::CursorGrabMode};
use bevy_egui::egui::{self, Response, Ui, Widget};

use crate::player::bindings::{self, Binding, Bindings};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorGrabbed>()
            .add_system(toggle_cursor_grab_system);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub struct CursorGrabbed(pub bool);

impl Default for CursorGrabbed {
    fn default() -> Self {
        Self(true)
    }
}

fn toggle_cursor_grab_system(
    mut window_query: Query<&mut Window>,
    mut cursor_grabbed: ResMut<CursorGrabbed>,
    bindings: Res<Bindings>,
    mut button_inputs: <bindings::Button as Binding>::Inputs<'_, '_>,
) {
    if bindings.toggle_cursor.just_pressed(&mut button_inputs) {
        let mut window = window_query.single_mut();
        if cursor_grabbed.0 {
            cursor_grabbed.0 = false;
            window.cursor.visible = true;
            window.cursor.grab_mode = CursorGrabMode::None;
        } else {
            cursor_grabbed.0 = true;
            window.cursor.visible = false;
            window.cursor.grab_mode = CursorGrabMode::Locked;
        }
    }
}

pub struct Vec3Widget<'a>(pub &'a mut Vec3);

impl<'a> Widget for Vec3Widget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut self.0.x));
            ui.add(egui::DragValue::new(&mut self.0.y));
            ui.add(egui::DragValue::new(&mut self.0.z));
        })
        .response
    }
}
