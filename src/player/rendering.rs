use bevy::prelude::*;

use crate::terrain::storage::Atoms;

use super::{LookPos, Player, PlayerUpdateSet};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_look_pos_marker_system.after(PlayerUpdateSet::TargetPos),
        );
    }
}
fn update_look_pos_marker_system(
    mut gizmos: Gizmos,
    player_query: Query<(&LookPos, &Transform), With<Player>>,
    world: Res<Atoms>,
) {
    let (look_pos, transform) = player_query.single();
    if let Some(look_pos) = look_pos.0 {
        let grid_pos = look_pos.grid_pos.as_vec3();
        gizmos.rect(
            grid_pos + look_pos.side.normal() * 0.504,
            look_pos.side.rotation(),
            Vec2::ONE,
            Color::BLACK,
        );
        if world.contains_atom(look_pos.grid_pos) {
            gizmos.cuboid(
                Transform::from_translation(grid_pos).with_scale(Vec3::splat(1.005)),
                Color::rgba(0.0, 0.0, 0.0, 0.33),
            );
        }
    }
    gizmos.rect(
        transform.translation + transform.forward() * 0.12,
        transform.rotation,
        Vec2::splat(0.0005),
        Color::BLACK,
    );
}
