use bevy::prelude::*;

use crate::{
    atom_physics::{element::Element, id::IdMap},
    terrain::storage::Atoms,
};

use super::{
    bindings::{self, Binding, Bindings},
    LookPos, Player, PlayerUpdateSet, SelectedElement,
};

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, place_atom_system.after(PlayerUpdateSet::Move))
            .insert_resource(SelectedElement(1));
    }
}

pub fn place_atom_system(
    mut world: ResMut<Atoms>,
    player_query: Query<&LookPos, With<Player>>,
    bindings: Res<Bindings>,
    mut inputs: <bindings::Button as Binding>::Inputs<'_, '_>,
    elements: Res<IdMap<Element>>,
    selected_element: Res<SelectedElement>,
) {
    let look_pos = player_query.single();
    if let Some(pos) = &look_pos.0 {
        if bindings.break_atom.just_pressed(&mut inputs) && world.contains_atom(pos.grid_pos) {
            world.set(pos.grid_pos.as_uvec3(), elements.air());
        }
        let place_pos = pos.grid_pos + pos.side.normal_ivec();
        if bindings.place_atom.just_pressed(&mut inputs) && world.contains_atom(place_pos) {
            if let Some(atom) = elements.instance_of(selected_element.0) {
                world.set(place_pos.as_uvec3(), atom);
            }
        }
    }
}
