use crate::spaceship::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

pub mod prelude {
    pub use super::SpaceshipCameraControlMode;
    pub use super::SpaceshipCameraControllerMarker;
    pub use super::SpaceshipCameraControllerPlugin;
    pub use super::SpaceshipCameraFreeLookInputMarker;
    pub use super::SpaceshipCameraInputMarker;
    pub use super::SpaceshipCameraNormalInputMarker;
    pub use super::SpaceshipCameraTurretInputMarker;
    pub use super::SpaceshipRotationInputActiveMarker;
}

pub struct SpaceshipCameraControllerPlugin;

impl Plugin for SpaceshipCameraControllerPlugin {
    fn build(&self, app: &mut App) {
        debug!("SpaceshipCameraControllerPlugin: build");

        app.init_resource::<SpaceshipCameraControlMode>();

        app.add_observer(insert_camera_controller);
        app.add_observer(insert_camera_freelook);
        app.add_observer(insert_camera_turret);

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
