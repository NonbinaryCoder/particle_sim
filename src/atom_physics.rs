use bevy::prelude::*;

mod inspector;
pub mod io;

pub struct AtomPhysicsPlugin;

impl Plugin for AtomPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((inspector::InspectorPlugin, io::IoPlugin));
    }
}
