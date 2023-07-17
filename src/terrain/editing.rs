use bevy::prelude::*;

use crate::player::{LookPos, Player, PlayerUpdateSet};

use super::{color::AtomColor, storage::Atoms, Atom};

pub struct EditingPlugin;

impl Plugin for EditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, place_atom_system.after(PlayerUpdateSet::Move));
    }
}

pub fn place_atom_system(
    mut world: ResMut<Atoms>,
    player_query: Query<&LookPos, With<Player>>,
    keys: Res<Input<KeyCode>>,
) {
    let look_pos = player_query.single();
    if let Some(pos) = &look_pos.0 {
        if keys.just_pressed(KeyCode::Q) {
            let pos = pos.grid_pos.as_uvec3();
            if world.contains_atom(pos) {
                world.set(pos, Atom::AIR);
            }
        }

        let place_pos = (pos.grid_pos + pos.side.normal_ivec()).as_uvec3();
        if world.contains_atom(place_pos) {
            if keys.just_pressed(KeyCode::E) {
                world.set(
                    place_pos,
                    Atom {
                        color: AtomColor::WHITE,
                    },
                );
            } else if keys.just_pressed(KeyCode::Z) {
                world.set(
                    place_pos,
                    Atom {
                        color: AtomColor::from_u32(0xff0000ff),
                    },
                );
            } else if keys.just_pressed(KeyCode::X) {
                world.set(
                    place_pos,
                    Atom {
                        color: AtomColor::from_u32(0x00ff00ff),
                    },
                );
            } else if keys.just_pressed(KeyCode::R) {
                world.set(
                    place_pos,
                    Atom {
                        color: AtomColor::from_u32(0x0000ff99),
                    },
                )
            }
        }
    }
}
