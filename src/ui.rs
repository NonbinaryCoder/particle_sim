use std::fmt::Display;

use bevy::{prelude::*, window::CursorGrabMode};
use bevy_egui::egui::{self, emath::Numeric, Response, Ui, Widget};

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

pub struct ArrayWidget<'a, T, const N: usize>(pub &'a [T; N]);

impl<'a, T: Display, const N: usize> Widget for ArrayWidget<'a, T, N> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            for val in self.0 {
                ui.label(val.to_string());
            }
        })
        .response
    }
}

pub struct ArrayMutWidget<'a, T, const N: usize>(pub &'a mut [T; N]);

impl<'a, T: Numeric, const N: usize> Widget for ArrayMutWidget<'a, T, N> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            for val in self.0 {
                ui.add(egui::DragValue::new(val));
            }
        })
        .response
    }
}
