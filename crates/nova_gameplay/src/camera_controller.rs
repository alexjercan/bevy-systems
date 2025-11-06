use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::prelude::*;

pub mod prelude {
    pub use super::{
        SpaceshipCameraControlMode, SpaceshipCameraControllerMarker,
        SpaceshipCameraControllerPlugin, SpaceshipCameraFreeLookInputMarker,
        SpaceshipCameraInputMarker, SpaceshipCameraNormalInputMarker,
        SpaceshipCameraTurretInputMarker, SpaceshipRotationInputActiveMarker,
    };
}

pub struct SpaceshipCameraControllerPlugin;

impl Plugin for SpaceshipCameraControllerPlugin {
    fn build(&self, app: &mut App) {
        debug!("SpaceshipCameraControllerPlugin: build");

        app.init_resource::<SpaceshipCameraControlMode>();
        app.add_input_context::<PlayerInputMarker>();

        app.add_observer(insert_camera_controller);
        app.add_observer(insert_camera_freelook);
        app.add_observer(insert_camera_turret);
        app.add_observer(insert_player_input);

        app.add_observer(on_rotation_input);
        app.add_observer(on_rotation_input_completed);
        app.add_observer(on_free_mode_input_started);
        app.add_observer(on_free_mode_input_completed);
        app.add_observer(on_combat_input_started);
        app.add_observer(on_combat_input_completed);

        app.add_systems(
            Update,
            (update_chase_camera_input, sync_spaceship_control_mode)
                .in_set(SpaceshipSystems::Camera),
        );
    }
}

/// Marker component to identify the camera controller for the player's spaceship.
///
/// This should be added to an entity that also has a `ChaseCamera` component.
#[derive(Component, Debug, Clone, Reflect)]
#[require(ChaseCamera)]
pub struct SpaceshipCameraControllerMarker;

/// The mode that the camera is currently in for controlling the spaceship.
#[derive(Resource, Default, Clone, Debug)]
pub enum SpaceshipCameraControlMode {
    #[default]
    Normal,
    FreeLook,
    Turret,
}

/// General Marker for the rotation input of the spaceship camera.
#[derive(Component, Debug, Clone)]
pub struct SpaceshipCameraInputMarker;

/// Marker for the rotation input of the spaceship camera in normal mode.
#[derive(Component, Debug, Clone)]
pub struct SpaceshipCameraNormalInputMarker;

/// Marker for the rotation input of the spaceship camera in free look mode.
#[derive(Component, Debug, Clone)]
pub struct SpaceshipCameraFreeLookInputMarker;

/// Marker for the rotation input of the spaceship camera in turret mode.
#[derive(Component, Debug, Clone)]
pub struct SpaceshipCameraTurretInputMarker;

#[derive(Component, Debug, Clone)]
pub struct SpaceshipRotationInputActiveMarker;

fn insert_camera_controller(
    add: On<Add, SpaceshipCameraControllerMarker>,
    mut commands: Commands,
    q_camera: Query<Entity, (With<ChaseCamera>, With<SpaceshipCameraControllerMarker>)>,
) {
    let entity = add.entity;
    trace!("insert_camera_controller: entity {:?}", entity);

    let Ok(camera) = q_camera.get(entity) else {
        warn!(
            "insert_camera_controller: entity {:?} not found in q_camera",
            add.entity
        );
        return;
    };

    commands.entity(camera).with_children(|parent| {
        parent.spawn((
            SpaceshipCameraInputMarker,
            SpaceshipCameraNormalInputMarker,
            SpaceshipRotationInputActiveMarker,
            PointRotation::default(),
        ));
    });
}

fn insert_camera_freelook(
    add: On<Add, SpaceshipCameraControllerMarker>,
    mut commands: Commands,
    q_camera: Query<Entity, (With<ChaseCamera>, With<SpaceshipCameraControllerMarker>)>,
) {
    let entity = add.entity;
    trace!("insert_camera_controller: entity {:?}", entity);

    let Ok(camera) = q_camera.get(entity) else {
        warn!(
            "insert_camera_controller: entity {:?} not found in q_camera",
            entity
        );
        return;
    };

    commands.entity(camera).with_children(|parent| {
        parent.spawn((
            SpaceshipCameraInputMarker,
            SpaceshipCameraFreeLookInputMarker,
            PointRotation::default(),
        ));
    });
}

fn insert_camera_turret(
    add: On<Add, SpaceshipCameraControllerMarker>,
    mut commands: Commands,
    q_camera: Query<Entity, (With<ChaseCamera>, With<SpaceshipCameraControllerMarker>)>,
) {
    let entity = add.entity;
    trace!("insert_camera_turret: entity {:?}", entity);

    let Ok(camera) = q_camera.get(entity) else {
        warn!(
            "insert_camera_turret: entity {:?} not found in q_camera",
            entity
        );
        return;
    };

    commands.entity(camera).with_children(|parent| {
        parent.spawn((
            SpaceshipCameraInputMarker,
            SpaceshipCameraTurretInputMarker,
            PointRotation::default(),
        ));
    });
}

fn insert_player_input(
    add: On<Add, SpaceshipCameraControllerMarker>,
    mut commands: Commands,
    q_camera: Query<Entity, (With<ChaseCamera>, With<SpaceshipCameraControllerMarker>)>,
) {
    let entity = add.entity;
    trace!("insert_camera_turret: entity {:?}", entity);

    let Ok(camera) = q_camera.get(entity) else {
        warn!(
            "insert_player_input: entity {:?} not found in q_camera",
            entity
        );
        return;
    };

    // Spawn a player input controller entity to hold the input from the player
    commands.entity(camera).with_children(|parent| {
        parent.spawn((
            Name::new("Player Input Controller"),
            PlayerInputMarker,
            actions!(
                PlayerInputMarker[
                    (
                        Action::<CameraInputRotate>::new(),
                        Bindings::spawn((
                            // Bevy requires single entities to be wrapped in `Spawn`.
                            // You can attach modifiers to individual bindings as well.
                            Spawn((Binding::mouse_motion(), Scale::splat(0.001), Negate::all())),
                            Axial::right_stick().with((Scale::splat(2.0), Negate::none())),
                        )),
                    ),
                    (
                        Action::<FreeLookInput>::new(),
                        bindings![KeyCode::AltLeft, GamepadButton::LeftTrigger],
                    ),
                    (
                        Action::<CombatInput>::new(),
                        bindings![MouseButton::Right],
                    ),
                ]
            ),
        ));
    });
}

fn update_chase_camera_input(
    camera: Single<
        &mut ChaseCameraInput,
        (With<ChaseCamera>, With<SpaceshipCameraControllerMarker>),
    >,
    spaceship: Single<&Transform, (With<SpaceshipRootMarker>, With<PlayerSpaceshipMarker>)>,
    point_rotation: Single<
        &PointRotationOutput,
        (
            With<SpaceshipCameraInputMarker>,
            With<SpaceshipRotationInputActiveMarker>,
        ),
    >,
) {
    // NOTE: We assume that only one of the input markers is active at a time.
    // We also assume that the spaceship and camera are singletons for the player.
    let mut camera_input = camera.into_inner();
    let spaceship_transform = spaceship.into_inner();
    let point_rotation = point_rotation.into_inner();

    camera_input.anchor_pos = spaceship_transform.translation;
    camera_input.achor_rot = **point_rotation;
}

fn sync_spaceship_control_mode(
    mut commands: Commands,
    mode: Res<SpaceshipCameraControlMode>,
    // NOTE: Just to ensure the spaceship exists
    _spaceship: Single<&Transform, (With<SpaceshipRootMarker>, With<PlayerSpaceshipMarker>)>,
    spaceship_input_rotation: Single<
        (Entity, &PointRotationOutput),
        With<SpaceshipCameraNormalInputMarker>,
    >,
    spaceship_input_free_look: Single<Entity, With<SpaceshipCameraFreeLookInputMarker>>,
    spaceship_input_turret: Single<Entity, With<SpaceshipCameraTurretInputMarker>>,
    camera: Single<Entity, (With<ChaseCamera>, With<SpaceshipCameraControllerMarker>)>,
) {
    if !mode.is_changed() {
        return;
    }

    // NOTE: We assume that only one of the input markers is active at a time.
    let (spaceship_input_rotation, point_rotation) = spaceship_input_rotation.into_inner();
    let spaceship_input_free_look = spaceship_input_free_look.into_inner();
    let spaceship_input_combat = spaceship_input_turret.into_inner();
    let camera = camera.into_inner();

    match *mode {
        SpaceshipCameraControlMode::Normal => {
            commands
                .entity(spaceship_input_rotation)
                .insert(SpaceshipRotationInputActiveMarker);
            commands
                .entity(spaceship_input_free_look)
                .remove::<SpaceshipRotationInputActiveMarker>();
            commands
                .entity(spaceship_input_combat)
                .remove::<SpaceshipRotationInputActiveMarker>();
            commands.entity(camera).insert(ChaseCamera {
                offset: Vec3::new(0.0, 5.0, -20.0),
                focus_offset: Vec3::new(0.0, 0.0, 20.0),
                ..default()
            });
        }
        SpaceshipCameraControlMode::FreeLook => {
            commands
                .entity(spaceship_input_rotation)
                .remove::<SpaceshipRotationInputActiveMarker>();
            commands
                .entity(spaceship_input_free_look)
                .insert(PointRotation {
                    initial_rotation: **point_rotation,
                })
                .insert(SpaceshipRotationInputActiveMarker);
            commands
                .entity(spaceship_input_combat)
                .remove::<SpaceshipRotationInputActiveMarker>();
            commands.entity(camera).insert(ChaseCamera {
                offset: Vec3::new(0.0, 10.0, -30.0),
                focus_offset: Vec3::new(0.0, 0.0, 0.0),
                ..default()
            });
        }
        SpaceshipCameraControlMode::Turret => {
            commands
                .entity(spaceship_input_rotation)
                .remove::<SpaceshipRotationInputActiveMarker>();
            commands
                .entity(spaceship_input_free_look)
                .remove::<SpaceshipRotationInputActiveMarker>();
            commands
                .entity(spaceship_input_combat)
                .insert(PointRotation {
                    initial_rotation: **point_rotation,
                })
                .insert(SpaceshipRotationInputActiveMarker);
            commands.entity(camera).insert(ChaseCamera {
                offset: Vec3::new(0.0, 5.0, -10.0),
                focus_offset: Vec3::new(0.0, 0.0, 50.0),
                ..default()
            });
        }
    }
}

#[derive(Component, Debug, Clone)]
struct PlayerInputMarker;

#[derive(InputAction)]
#[action_output(Vec2)]
struct CameraInputRotate;

#[derive(InputAction)]
#[action_output(bool)]
struct FreeLookInput;

#[derive(InputAction)]
#[action_output(bool)]
struct CombatInput;

fn on_rotation_input(
    fire: On<Fire<CameraInputRotate>>,
    mut q_input: Query<
        &mut PointRotationInput,
        (
            With<SpaceshipCameraInputMarker>,
            With<SpaceshipRotationInputActiveMarker>,
        ),
    >,
) {
    for mut input in &mut q_input {
        **input = fire.value;
    }
}

fn on_rotation_input_completed(
    _: On<Complete<CameraInputRotate>>,
    mut q_input: Query<&mut PointRotationInput, With<SpaceshipCameraInputMarker>>,
) {
    for mut input in &mut q_input {
        **input = Vec2::ZERO;
    }
}

fn on_free_mode_input_started(
    _: On<Start<FreeLookInput>>,
    mut mode: ResMut<SpaceshipCameraControlMode>,
) {
    *mode = SpaceshipCameraControlMode::FreeLook;
}

fn on_free_mode_input_completed(
    _: On<Complete<FreeLookInput>>,
    mut mode: ResMut<SpaceshipCameraControlMode>,
) {
    *mode = SpaceshipCameraControlMode::Normal;
}

fn on_combat_input_started(
    _: On<Start<CombatInput>>,
    mut mode: ResMut<SpaceshipCameraControlMode>,
) {
    *mode = SpaceshipCameraControlMode::Turret;
}

fn on_combat_input_completed(
    _: On<Complete<CombatInput>>,
    mut mode: ResMut<SpaceshipCameraControlMode>,
) {
    *mode = SpaceshipCameraControlMode::Normal;
}
