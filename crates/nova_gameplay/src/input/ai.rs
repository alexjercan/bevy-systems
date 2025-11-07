use bevy::prelude::*;

use crate::prelude::*;

pub mod prelude {
    pub use super::{AISpaceshipMarker, SpaceshipAIInputPlugin};
}

pub struct SpaceshipAIInputPlugin;

impl Plugin for SpaceshipAIInputPlugin {
    fn build(&self, app: &mut App) {
        debug!("SpaceshipAIInputPlugin: build");

        app.add_systems(
            Update,
            (
                update_controller_target_rotation_torque,
                update_turret_target_input,
                on_thruster_input,
                on_projectile_input,
            )
                .in_set(SpaceshipSystems::Input),
        );
    }
}

/// Marker component to identify the ai's spaceship.
///
/// This should be added to the root entity of the ai's spaceship.
#[derive(Component, Debug, Clone, Reflect)]
#[require(SpaceshipRootMarker)]
pub struct AISpaceshipMarker;

fn update_controller_target_rotation_torque(
    mut q_controller: Query<
        (&mut ControllerSectionRotationInput, &ChildOf),
        With<ControllerSectionMarker>,
    >,
    q_spaceship: Query<(Entity, &Transform), (With<SpaceshipRootMarker>, With<AISpaceshipMarker>)>,
    player: Single<&Transform, (With<SpaceshipRootMarker>, With<PlayerSpaceshipMarker>)>,
) {
    let player_transform = player.into_inner();

    for (entity, spaceship_transform) in &q_spaceship {
        let direction_to_player =
            (player_transform.translation - spaceship_transform.translation).normalize();
        let target_rotation = Quat::from_rotation_arc(Vec3::NEG_Z, direction_to_player);

        for (mut controller, _) in q_controller
            .iter_mut()
            .filter(|(_, ChildOf(c_parent))| *c_parent == entity)
        {
            **controller = target_rotation;
        }
    }
}

fn update_turret_target_input(
    mut q_turret: Query<(&mut TurretSectionTargetInput, &ChildOf), With<TurretSectionMarker>>,
    q_spaceship: Query<Entity, (With<SpaceshipRootMarker>, With<AISpaceshipMarker>)>,
    player: Single<&Transform, (With<SpaceshipRootMarker>, With<PlayerSpaceshipMarker>)>,
) {
    // NOTE: We assume that there is only one player spaceship in the scene.
    let transform = player.into_inner();

    for entity in &q_spaceship {
        for (mut turret_input, _) in q_turret
            .iter_mut()
            .filter(|(_, ChildOf(c_parent))| *c_parent == entity)
        {
            **turret_input = Some(transform.translation);
        }
    }
}

fn on_thruster_input(
    mut q_thruster: Query<(&mut ThrusterSectionInput, &GlobalTransform, &ChildOf), With<ThrusterSectionMarker>>,
    q_spaceship: Query<(Entity, &Transform), (With<SpaceshipRootMarker>, With<AISpaceshipMarker>)>,
    player: Single<&Transform, (With<SpaceshipRootMarker>, With<PlayerSpaceshipMarker>)>,
) {
    let player_transform = player.into_inner();

    for (entity, spaceship_transform) in &q_spaceship {
        let to_player = player_transform.translation - spaceship_transform.translation;
        let direction_to_player = to_player.normalize();

        for (mut thruster_input, thruster_transform, _) in q_thruster
            .iter_mut()
            .filter(|(_, _, ChildOf(c_parent))| *c_parent == entity)
        {
            // TODO: consider using a more sophisticated method to determine thrust level
            let forward = thruster_transform.forward();
            let alignment = forward.dot(direction_to_player).clamp(-1.0, 1.0);

            **thruster_input = alignment.max(0.0);
        }
    }
}

fn on_projectile_input(
    mut commands: Commands,
    q_turret: Query<(Entity, &TurretSectionMuzzleEntity, &ChildOf), With<TurretSectionMarker>>,
    q_muzzle: Query<&GlobalTransform, With<TurretSectionBarrelMuzzleMarker>>,
    q_spaceship: Query<Entity, (With<SpaceshipRootMarker>, With<AISpaceshipMarker>)>,
    player: Single<&Transform, (With<SpaceshipRootMarker>, With<PlayerSpaceshipMarker>)>,
) {
    let player_transform = player.into_inner();

    for entity in &q_spaceship {
        for (turret, muzzle, _) in q_turret
            .iter()
            .filter(|(_, _, ChildOf(c_parent))| *c_parent == entity)
        {
            let Ok(muzzle_transform) = q_muzzle.get(**muzzle) else {
                warn!(
                    "on_projectile_input: muzzle entity {:?} not found in q_muzzle",
                    **muzzle
                );
                continue;
            };

            let direction_to_player =
                (player_transform.translation - muzzle_transform.translation()).normalize();
            let forward = muzzle_transform.forward();

            let alignment = forward.dot(direction_to_player);
            if alignment > 0.95 {
                commands.trigger(TurretShoot { entity: turret });
            }
        }
    }
}
