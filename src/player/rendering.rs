use bevy::prelude::*;

use super::{LookPos, Player, PlayerUpdateSet};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_look_pos_marker_system)
            .add_system(update_look_pos_marker_system.after(PlayerUpdateSet::TargetPos));
    }
}

/// In-world marker showing where the player is looking.
#[derive(Debug, Clone, Component)]
struct LookPosMarker;

type LookPosMarkerMaterial = StandardMaterial;

fn spawn_look_pos_marker_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LookPosMarkerMaterial>>,
) {
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(shape::Cube::new(1.0).into()),
            material: materials.add(LookPosMarkerMaterial::default()),
            transform: Transform::from_scale(Vec3::splat(0.125)),
            ..default()
        },
        LookPosMarker,
    ));
}

fn update_look_pos_marker_system(
    player_query: Query<&LookPos, With<Player>>,
    mut marker_query: Query<(&mut Transform, &mut Visibility), With<LookPosMarker>>,
) {
    let pos = player_query.single();
    let (mut transform, mut visibility) = marker_query.single_mut();
    if let Some(pos) = &pos.0 {
        *visibility = Visibility::Visible;
        transform.translation = pos.world;
    } else {
        *visibility = Visibility::Hidden;
    }
}
