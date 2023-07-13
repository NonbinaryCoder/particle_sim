//! Player controller.

use std::{f32::consts::PI, mem};

use bevy::prelude::*;

use crate::{
    terrain::{
        storage::{Atoms, RaycastHit},
        Direction,
    },
    ui::CursorGrabbed,
};

use self::{
    bindings::{Binding, Bindings},
    physics::{CollisionPoint, Rect3d},
};

pub mod bindings;
mod inspector;
mod physics;
mod rendering;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            inspector::InspectorPlugin,
            bindings::BindingsPlugin,
            rendering::RenderingPlugin,
        ))
        .insert_resource(PlayerConfig {
            speed: 0.5,
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
    freecam_enabled: bool,
}

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
    mut query: Query<(&mut Momentum, &Transform), With<ControlledEntity>>,
    config: Res<PlayerConfig>,
    bindings: Res<Bindings>,
    mut inputs: <bindings::Axis2 as Binding>::Inputs<'_, '_>,
) {
    // This system will need to be rewritten when a proper friction system is
    // added.
    fn flatten(v: Vec3) -> Vec3 {
        Vec3 { y: 0.0, ..v }
    }

    let (mut momentum, transform) = query.single_mut();
    let local_walk = bindings.walk.value_clamped(&mut inputs);
    let global_walk = flatten(transform.forward()) * local_walk.y
        + flatten(transform.left()) * local_walk.x
        + Vec3::Y * bindings.up_down.value_clamped(inputs.as_mut());
    momentum.0 += global_walk.normalize_or_zero() * config.speed;
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
) {
    let (mut look_pos, transform) = player_query.single_mut();

    let ray = Ray {
        origin: transform.translation,
        direction: transform.forward(),
    };

    let world_size = world.size().as_vec3();
    let extents_a = Vec3::X * world_size.x;
    let extents_b = Vec3::Z * world_size.z;
    let floor = Rect3d {
        origin: (extents_a + extents_b) * 0.5 - Vec3::splat(0.5),
        extents_a,
        extents_b,
    };

    look_pos.0 = world.raycast(ray, |atom| atom.is_visible());
}

fn wall_look_pos(
    floor: Rect3d,
    ray: Ray,
    world_size: Vec3,
    grid_size: UVec3,
) -> Option<(Vec3, IVec3, Direction)> {
    macro_rules! return_if_some {
        ($e:expr, $xyz:ident = $val:expr, $d:expr) => {
            match $e {
                Some(v) => {
                    return Some((
                        v,
                        {
                            let mut pos = v.round().as_ivec3();
                            pos.$xyz = $val;
                            pos
                        },
                        $d,
                    ))
                }
                None => (),
            }
        };
    }

    fn flip(mut rect: Rect3d, movement: Vec3) -> Rect3d {
        mem::swap(&mut rect.extents_a, &mut rect.extents_b);
        rect.origin += movement;
        rect
    }

    return_if_some!(
        flip(floor, Vec3::Y * world_size.y).collision_point(&ray),
        y = grid_size.y as i32,
        Direction::NegY
    );

    let extents_a = Vec3::Z * world_size.z;
    let extents_b = Vec3::Y * world_size.y;
    let wall_x = Rect3d {
        origin: (extents_a + extents_b) * 0.5 - Vec3::splat(0.5),
        extents_a,
        extents_b,
    };

    return_if_some!(wall_x.collision_point(&ray), x = -1, Direction::PosX);
    return_if_some!(
        flip(wall_x, Vec3::X * world_size.x).collision_point(&ray),
        x = grid_size.x as i32,
        Direction::NegX
    );

    let extents_a = Vec3::Y * world_size.y;
    let extents_b = Vec3::X * world_size.x;
    let wall_z = Rect3d {
        origin: (extents_a + extents_b) * 0.5 - Vec3::splat(0.5),
        extents_a,
        extents_b,
    };

    return_if_some!(wall_z.collision_point(&ray), z = -1, Direction::PosZ);
    return_if_some!(
        flip(wall_z, Vec3::Z * world_size.z).collision_point(&ray),
        z = grid_size.z as i32,
        Direction::NegZ
    );

    None
}
