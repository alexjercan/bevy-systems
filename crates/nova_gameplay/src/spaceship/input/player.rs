use crate::spaceship::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

pub mod prelude {
    pub use super::PlayerSpaceshipMarker;
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpaceshipPlayerInputPluginSet;

pub struct SpaceshipPlayerInputPlugin;

// TODO: Add some input for the thrusters and shooting, etc. here
impl Plugin for SpaceshipPlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_controller_target_rotation_torque,
                update_turret_target_input,
            )
                .in_set(SpaceshipPlayerInputPluginSet)
                .chain(),
        );
    }
}

/// Marker component to identify the player's spaceship.
///
/// This should be added to the root entity of the player's spaceship.
#[derive(Component, Debug, Clone, Reflect)]
#[require(SpaceshipRootMarker)]
pub struct PlayerSpaceshipMarker;

/// System that takes the point rotation output from the chase camera and applies it to the
/// controller of the player's spaceship.
fn update_controller_target_rotation_torque(
    point_rotation: Single<
        &PointRotationOutput,
        (
            With<SpaceshipCameraInputMarker>,
            With<SpaceshipCameraNormalInputMarker>,
        ),
    >,
    mut q_controller: Query<
        (&mut ControllerSectionRotationInput, &ChildOf),
        With<ControllerSectionMarker>,
    >,
    spaceship: Single<Entity, (With<SpaceshipRootMarker>, With<PlayerSpaceshipMarker>)>,
) {
    // NOTE: we assume that there is only one point rotation output with the given markers.
    let point_rotation = point_rotation.into_inner();
    // NOTE: We assume that there is only one player spaceship in the scene.
    let spaceship = spaceship.into_inner();

    for (mut controller, _) in q_controller
        .iter_mut()
        .filter(|(_, ChildOf(c_parent))| *c_parent == spaceship)
    {
        **controller = **point_rotation;
    }
}

/// System that takes the point rotation output from the chase camera and applies it to the
/// turret target input of the player's spaceship.
fn update_turret_target_input(
    point_rotation: Single<
        &PointRotationOutput,
        (
            With<SpaceshipCameraInputMarker>,
            With<SpaceshipCameraTurretInputMarker>,
        ),
    >,
    mut q_turret: Query<(&mut TurretSectionTargetInput, &ChildOf), With<TurretSectionMarker>>,
    spaceship: Single<
        (&Transform, Entity),
        (With<SpaceshipRootMarker>, With<PlayerSpaceshipMarker>),
    >,
) {
    // NOTE: we assume that there is only one point rotation output with the given markers.
    let point_rotation = point_rotation.into_inner();
    // NOTE: We assume that there is only one player spaceship in the scene.
    let (transform, spaceship) = spaceship.into_inner();

    for (mut turret, _) in q_turret
        .iter_mut()
        .filter(|(_, ChildOf(t_parent))| *t_parent == spaceship)
    {
        let forward = **point_rotation * -Vec3::Z;
        let position = transform.translation;
        let distance = 100.0;

        **turret = Some(position + forward * distance);
    }
}
