use std::fmt;

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

#[derive(Clone, Copy)]
pub struct PrettyPrint<'a>(&'a [u8]);

impl<'a> fmt::Display for PrettyPrint<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match std::str::from_utf8(self.0) {
            Ok(s) => write!(f, "{s}"),
            Err(_) => write!(f, "{:?}", self.0),
        }
    }
}

impl<'a> fmt::Debug for PrettyPrint<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match std::str::from_utf8(self.0) {
            Ok(s) => write!(f, "{s:?}"),
            Err(_) => f.debug_list().entries(self.0).finish(),
        }
    }
}
