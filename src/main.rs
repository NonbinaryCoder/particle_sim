use bevy::{prelude::*, window::CursorGrabMode, DefaultPlugins};
use bevy_egui::EguiPlugin;

mod player;
mod terrain;
mod ui;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin,
            terrain::TerrainPlugin,
            player::PlayerPlugin,
            ui::UiPlugin,
        ))
        .add_systems(Startup, setup_window_system)
        .run();
}

/// Setup system that sets window title and hides and grabs the cursor.
fn setup_window_system(mut window_query: Query<&mut Window>) {
    let mut window = window_query.single_mut();
    if cfg!(debug_assertions) {
        window.title = "Particle Sim (DEBUG BUILD)".to_string();
    } else {
        window.title = "Particle Sim".to_string();
    }
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;
}
