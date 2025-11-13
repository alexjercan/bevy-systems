use avian3d::prelude::*;
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
                on_thruster_input,
                update_turret_target_input,
                on_projectile_input,
            )
                .in_set(super::SpaceshipInputSystems),
        );
    }
}

/// Marker component to identify the ai's spaceship.
///
/// This should be added to the root entity of the ai's spaceship.
#[derive(Component, Debug, Clone, Reflect)]
#[require(SpaceshipRootMarker)]
pub struct AISpaceshipMarker;

// NOTE: The AI was generated using ChatGPT to see how good it can play my game :)

fn update_controller_target_rotation_torque(
    mut q_controller: Query<
        (&mut ControllerSectionRotationInput, &ChildOf),
        With<ControllerSectionMarker>,
    >,
    q_spaceship: Query<
        (Entity, &Transform, &LinearVelocity),
        (With<SpaceshipRootMarker>, With<AISpaceshipMarker>),
    >,
    player: Single<&Transform, (With<SpaceshipRootMarker>, With<PlayerSpaceshipMarker>)>,
) {
    let player_transform = player.into_inner();

    for (entity, transform, velocity) in &q_spaceship {
        let to_player = player_transform.translation - transform.translation;
        let distance = to_player.length();
        let speed = velocity.length();

        // Determine whether to chase or brake
        let target_speed = (distance * 0.2).clamp(2.0, 20.0);
        let too_fast = speed > target_speed + 1.0;

        // Desired direction:
        // - toward player if slow
        // - opposite of velocity if too fast
        let desired_direction = if too_fast {
            // Brake
            -velocity.normalize_or_zero()
        } else {
            // Chase
            to_player.normalize()
        };

        // If velocity is zero (e.g., stationary), fall back to facing player
        let desired_direction = if desired_direction.length_squared() == 0.0 {
            to_player.normalize_or_zero()
        } else {
            desired_direction
        };

        let forward = transform.forward().into();
        let target_rotation = Quat::from_rotation_arc(forward, desired_direction);

        for (mut controller, _) in q_controller
            .iter_mut()
            .filter(|(_, ChildOf(parent))| *parent == entity)
        {
            **controller = target_rotation;
        }
    }
}

fn on_thruster_input(
    mut q_thruster: Query<
        (&mut ThrusterSectionInput, &GlobalTransform, &ChildOf),
        With<ThrusterSectionMarker>,
    >,
    q_spaceship: Query<
        (Entity, &Transform, &LinearVelocity),
        (With<SpaceshipRootMarker>, With<AISpaceshipMarker>),
    >,
    player: Single<&Transform, (With<SpaceshipRootMarker>, With<PlayerSpaceshipMarker>)>,
) {
    let player_transform = player.into_inner();

    for (entity, transform, velocity) in &q_spaceship {
        let to_player = player_transform.translation - transform.translation;
        let distance = to_player.length();
        let speed = velocity.length();

        let target_speed = (distance * 0.2).clamp(2.0, 20.0);
        let too_fast = speed > target_speed + 1.0;

        let desired_direction = if too_fast {
            -velocity.normalize_or_zero()
        } else {
            to_player.normalize()
        };

        // Determine how well we’re aligned before applying thrust
        let forward = transform.forward();
        let alignment = forward.dot(desired_direction);

        // Apply thrust only if pointing in roughly the correct direction
        // and not already moving too fast
        let should_thrust = alignment > 0.95;

        let thrust_level = if should_thrust { 1.0 } else { 0.0 };

        for (mut thruster_input, _, _) in q_thruster
            .iter_mut()
            .filter(|(_, _, ChildOf(parent))| *parent == entity)
        {
            **thruster_input = thrust_level;
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
                error!(
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
