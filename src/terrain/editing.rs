use bevy::prelude::*;

use crate::player::{LookPos, Player, PlayerUpdateSet};

use super::{color::AtomColor, storage::Atoms, Atom};

pub struct EditingPlugin;

impl Plugin for EditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(place_atom_system.after(PlayerUpdateSet::Move));
    }
}

pub fn place_atom_system(
    mut world: ResMut<Atoms>,
    player_query: Query<&LookPos, With<Player>>,
    keys: Res<Input<KeyCode>>,
) {
    let look_pos = player_query.single();
    if let Some(pos) = &look_pos.0 {
        let pos = (pos.grid + pos.direction.normal_ivec()).as_uvec3();
        if world.contains_pos(pos) {
            if keys.just_pressed(KeyCode::Q) {
                world.set(pos, Atom::default());
            } else if keys.just_pressed(KeyCode::E) {
                world.set(
                    pos,
                    Atom {
                        color: AtomColor::WHITE,
                    },
                );
            } else if keys.just_pressed(KeyCode::Z) {
                world.set(
                    pos,
                    Atom {
                        color: AtomColor::from_u32(0xff0000ff),
                    },
                );
            } else if keys.just_pressed(KeyCode::X) {
                world.set(
                    pos,
                    Atom {
                        color: AtomColor::from_u32(0x00ff00ff),
                    },
                );
            }
        }
    }
}
