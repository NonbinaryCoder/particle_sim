//! Player controller.

use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    atom_physics::element::ElementId,
    terrain::storage::{Atoms, RaycastHit},
    ui::CursorGrabbed,
};

use self::bindings::{Binding, Bindings};

pub mod bindings;
mod building;
mod inspector;
mod rendering;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            bindings::BindingsPlugin,
            building::BuildingPlugin,
            inspector::InspectorPlugin,
            rendering::RenderingPlugin,
        ))
        .insert_resource(PlayerConfig {
            speed: 0.5,
            reach_dist: 32.0,
            freecam_enabled: false,
        })
        .add_systems(Startup, spawn_player_system)
        .add_systems(
            Update,
            (
                player_look_system.run_if(resource_equals(CursorGrabbed(true))),
                look_direction_system,
            )
                .chain()
                .in_set(PlayerUpdateSet::Look),
        )
        .add_systems(
            Update,
            (player_move_system, apply_momentum_system)
                .chain()
                .in_set(PlayerUpdateSet::Move)
                .after(PlayerUpdateSet::Look),
        )
        .add_systems(
            Update,
            (
                apply_friction_system,
                player_look_pos_system.in_set(PlayerUpdateSet::TargetPos),
            )
                .after(PlayerUpdateSet::Move),
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum PlayerUpdateSet {
    Look,
    Move,
    TargetPos,
}

#[derive(Debug, Clone, Resource)]
pub struct PlayerConfig {
    speed: f32,
    reach_dist: f32,
    freecam_enabled: bool,
}

#[derive(Debug, Clone, Copy, Resource)]
pub struct SelectedElement(pub ElementId);

/// Marker component for the main player.
#[derive(Debug, Component)]
pub struct Player;

/// Marker component for what entity controls effect.  Different from `Player`
/// component because of freecam.
#[derive(Debug, Component)]
pub struct ControlledEntity;

/// Startup system that spawns the player camera.
fn spawn_player_system(mut commands: Commands) {
    commands.spawn((
        Player,
        ControlledEntity,
        Camera3dBundle::default(),
        LookDirection::default(),
        Momentum::default(),
        LookPos::default(),
    ));
}

/// What direction an entity is looking.
#[derive(Debug, Default, Clone, Copy, Component)]
struct LookDirection {
    /// 0 is directly ahead, π/2 is directly up, -π/2 is directly down.
    vertical: f32,
    /// Ranges between 0 and 2π
    horizontal: f32,
}

/// Main schedule system that updates what direction the player is looking in
/// based on inputs.
fn player_look_system(
    mut query: Query<&mut LookDirection, With<ControlledEntity>>,
    bindings: Res<Bindings>,
    mut inputs: <bindings::Axis2 as Binding>::Inputs<'_, '_>,
) {
    if let Ok(mut look_direction) = query.get_single_mut() {
        let delta = bindings.look.value(&mut inputs);
        look_direction.vertical = (look_direction.vertical - delta.y).clamp(-PI * 0.5, PI * 0.5);
        look_direction.horizontal = (look_direction.horizontal - delta.x).rem_euclid(PI * 2.0);
    }
}

/// Updates entities with a [`LookDirection`] component to look in that
/// direction.
fn look_direction_system(
    mut query: Query<(&LookDirection, &mut Transform), Changed<LookDirection>>,
) {
    for (look_direction, mut transform) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_y(look_direction.horizontal);
        transform.rotate_local_x(look_direction.vertical);
    }
}

#[derive(Debug, Default, Clone, Copy, Component)]
struct Momentum(Vec3);

fn player_move_system(
    mut query: Query<(&mut Momentum, &LookDirection), With<ControlledEntity>>,
    config: Res<PlayerConfig>,
    bindings: Res<Bindings>,
    mut inputs: <bindings::Axis2 as Binding>::Inputs<'_, '_>,
) {
    // This system will need to be rewritten when a proper friction system is
    // added.
    let local_walk = bindings.walk.value_clamped(&mut inputs);
    let local_up_down = bindings.up_down.value_clamped(inputs.as_mut());
    if local_walk.length_squared() >= 0.001 || local_up_down.abs() >= 0.001 {
        let (mut momentum, look_dir) = query.single_mut();

        let rotation = Quat::from_rotation_y(look_dir.horizontal);
        let forward = rotation * -Vec3::Z;
        let left = rotation * -Vec3::X;

        let global_walk = forward * local_walk.y + left * local_walk.x + Vec3::Y * local_up_down;
        momentum.0 += global_walk.normalize_or_zero() * config.speed;
    }
}

fn apply_momentum_system(mut query: Query<(&Momentum, &mut Transform)>) {
    for (momentum, mut transform) in query.iter_mut() {
        transform.translation += momentum.0;
    }
}

fn apply_friction_system(mut query: Query<&mut Momentum>) {
    for mut momentum in query.iter_mut() {
        // Will need to change with proper friction system.
        momentum.0 = Vec3::ZERO;
    }
}

/// The atom and world pos the player is currently looking at.
#[derive(Debug, Default, Clone, Component)]
pub struct LookPos(pub Option<RaycastHit>);

/// Updates information about what atom the player is currently looking at.
fn player_look_pos_system(
    world: Res<Atoms>,
    mut player_query: Query<(&mut LookPos, &Transform), With<Player>>,
    config: Res<PlayerConfig>,
) {
    let (mut look_pos, transform) = player_query.single_mut();

    let ray = Ray {
        origin: transform.translation,
        direction: transform.forward(),
    };

    look_pos.0 = world.raycast(ray, config.reach_dist, |atom| atom.is_visible());
}
