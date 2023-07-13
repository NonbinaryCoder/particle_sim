use bevy::{
    prelude::*,
    render::render_resource::encase::vector::{AsMutVectorParts, AsRefVectorParts},
};
use bevy_egui::{
    egui::{self, DragValue},
    EguiContexts,
};

use crate::ui::{ArrayMutWidget, ArrayWidget};

use super::{ControlledEntity, LookDirection, LookPos, Momentum, Player, PlayerConfig};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, player_inspector_system)
            .add_systems(
                Update,
                player_gizmo_system.run_if(|config: Res<PlayerConfig>| config.freecam_enabled),
            );
    }
}

fn player_inspector_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut player_query: Query<
        (Entity, &mut Transform, &Momentum, &LookDirection, &LookPos),
        With<Player>,
    >,
    controlled_query: Query<Entity, (With<ControlledEntity>, Without<Player>)>,
    mut config: ResMut<PlayerConfig>,
) {
    let (player_id, mut transform, momentum, look_direction, look_pos) = player_query.single_mut();
    egui::Window::new("Player Inspector")
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label("Position:");
            ui.add(ArrayMutWidget(transform.translation.as_mut_parts()));

            ui.label("Looking:");
            ui.add(ArrayWidget(round(transform.forward(), 4).as_ref_parts()));

            if let Some(look_pos) = &look_pos.0 {
                ui.label("Target:");
                ui.add(ArrayWidget(look_pos.grid_pos.as_ref_parts()));
            }

            ui.horizontal(|ui| {
                ui.label("Speed: ");
                ui.add(DragValue::new(&mut config.speed).speed(0.1));
            });

            let mut freecam = config.freecam_enabled;
            ui.checkbox(&mut freecam, "Freecam");
            if freecam != config.freecam_enabled {
                config.freecam_enabled = freecam;
                if freecam {
                    commands.spawn((
                        ControlledEntity,
                        Camera3dBundle {
                            transform: *transform,
                            ..default()
                        },
                        *momentum,
                        *look_direction,
                    ));
                    commands
                        .entity(player_id)
                        .remove::<(ControlledEntity, Camera)>();
                } else {
                    commands
                        .entity(player_id)
                        .insert((ControlledEntity, Camera3dBundle::default().camera));
                    commands.entity(controlled_query.single()).despawn();
                }
            }
        });
}

fn round(val: Vec3, places: u32) -> Vec3 {
    let v = 10.0_f32.powi(places as i32);
    (val * v).round() / v
}

fn player_gizmo_system(player_query: Query<&Transform, With<Player>>, mut gizmos: Gizmos) {
    gizmos
        .sphere(
            player_query.single().translation,
            Quat::IDENTITY,
            0.125,
            Color::BLACK,
        )
        .circle_segments(4);
}
