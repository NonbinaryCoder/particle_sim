use bevy::prelude::*;

use self::id::MappedToId;

pub mod element;
pub mod id;
mod inspector;
pub mod io;
mod value;

pub struct AtomPhysicsPlugin;

impl Plugin for AtomPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((inspector::InspectorPlugin, io::IoPlugin))
            .insert_resource(element::Element::create_map());
    }
}
