use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::prelude::*;

pub mod prelude {
    pub use super::{
        PlayerSpaceshipMarker, SpaceshipPlayerInputPlugin, SpaceshipThrusterInputBinding,
        SpaceshipTurretInputBinding,
    };
}

pub struct SpaceshipPlayerInputPlugin;

impl Plugin for SpaceshipPlayerInputPlugin {
    fn build(&self, app: &mut App) {
        debug!("SpaceshipPlayerInputPlugin: build");

        app.add_input_context::<ThrusterInputMarker>();
        app.add_observer(on_thruster_input_binding);
        app.add_observer(on_thruster_input);
        app.add_observer(on_thruster_input_completed);

        app.add_input_context::<TurretInputMarker>();
        app.add_observer(on_turret_input_binding);
        app.add_observer(on_projectile_input);
        app.add_observer(on_projectile_input_completed);

        app.add_systems(
            Update,
            (
                update_controller_target_rotation_torque,
                update_turret_target_input,
            )
                .in_set(super::SpaceshipInputSystems),
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
    let point_rotation = point_rotation.into_inner();
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
    let point_rotation = point_rotation.into_inner();
    let (transform, spaceship) = spaceship.into_inner();

    for (mut turret, _) in q_turret
        .iter_mut()
        .filter(|(_, ChildOf(t_parent))| *t_parent == spaceship)
    {
        let forward = **point_rotation * Vec3::NEG_Z;
        let position = transform.translation;
        let distance = 100.0;

        **turret = Some(position + forward * distance);
    }
}

#[derive(Component, Debug, Clone, Deref, DerefMut, Reflect)]
pub struct SpaceshipThrusterInputBinding(pub Vec<Binding>);

#[derive(Component, Debug, Clone)]
struct ThrusterInputMarker;

#[derive(InputAction)]
#[action_output(bool)]
struct ThrusterInput;

fn on_thruster_input_binding(
    add: On<Add, SpaceshipThrusterInputBinding>,
    mut commands: Commands,
    q_binding: Query<&SpaceshipThrusterInputBinding>,
) {
    let entity = add.entity;
    trace!("on_thruster_input_binding: entity {:?}", entity);

    let Ok(binding) = q_binding.get(entity) else {
        error!(
            "on_thruster_input_binding: entity {:?} not found in q_binding",
            entity
        );
        return;
    };

    commands.entity(entity).insert((
        ThrusterInputMarker,
        actions!(
            ThrusterInputMarker[(
                Name::new("Input: Thruster"),
                Action::<ThrusterInput>::new(),
                ActionSettings {
                    consume_input: false,
                    ..default()
                },
                Bindings::spawn(binding.0.clone()),
            )]
        ),
    ));
}

fn on_thruster_input(
    fire: On<Start<ThrusterInput>>,
    mut q_input: Query<&mut ThrusterSectionInput, With<ThrusterInputMarker>>,
) {
    let entity = fire.event().context;
    trace!("on_thruster_input: entity {:?}", entity);

    let Ok(mut q_input) = q_input.get_mut(entity) else {
        error!(
            "on_thruster_input: entity {:?} not found in q_input",
            entity
        );
        return;
    };

    **q_input = 1.0;
}

fn on_thruster_input_completed(
    fire: On<Complete<ThrusterInput>>,
    mut q_input: Query<&mut ThrusterSectionInput, With<ThrusterInputMarker>>,
) {
    let entity = fire.event().context;
    trace!("on_thruster_input_completed: entity {:?}", entity);

    let Ok(mut q_input) = q_input.get_mut(entity) else {
        return;
    };

    **q_input = 0.0;
}

#[derive(Component, Debug, Clone, Deref, DerefMut, Reflect)]
pub struct SpaceshipTurretInputBinding(pub Vec<Binding>);

#[derive(Component, Debug, Clone)]
struct TurretInputMarker;

#[derive(InputAction)]
#[action_output(bool)]
struct TurretInput;

fn on_turret_input_binding(
    add: On<Add, SpaceshipTurretInputBinding>,
    mut commands: Commands,
    q_binding: Query<&SpaceshipTurretInputBinding>,
) {
    let entity = add.entity;
    trace!("on_turret_input_binding: entity {:?}", entity);

    let Ok(binding) = q_binding.get(entity) else {
        return;
    };

    commands.entity(entity).insert((
        TurretInputMarker,
        actions!(
            TurretInputMarker[(
                Name::new("Input: Turret"),
                Action::<TurretInput>::new(),
                ActionSettings {
                    consume_input: false,
                    ..default()
                },
                Bindings::spawn(binding.0.clone()),
            )]
        ),
    ));
}

fn on_projectile_input(
    fire: On<Start<TurretInput>>,
    mut q_turret: Query<&mut TurretSectionInput, With<TurretInputMarker>>,
) {
    let entity = fire.event().context;
    trace!("on_projectile_input: entity {:?}", entity);

    let Ok(mut q_turret) = q_turret.get_mut(entity) else {
        return;
    };

    **q_turret = true;
}

fn on_projectile_input_completed(
    fire: On<Complete<TurretInput>>,
    mut q_turret: Query<&mut TurretSectionInput, With<TurretInputMarker>>,
) {
    let entity = fire.event().context;
    trace!("on_projectile_input_completed: entity {:?}", entity);

    let Ok(mut q_turret) = q_turret.get_mut(entity) else {
        return;
    };

    **q_turret = false;
}
