use bevy::{prelude::*, window::CursorGrabMode, DefaultPlugins};
use bevy_egui::EguiPlugin;

mod player;
mod terrain;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(terrain::TerrainPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(ui::UiPlugin)
        .add_startup_system(setup_window_system)
        .run();
}

/// Setup system that sets window title and hides and grabs the cursor.
fn setup_window_system(mut window_query: Query<&mut Window>) {
    let mut window = window_query.single_mut();
    window.title = "Particle Sim".to_string();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;
}
