use crate::spaceship::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

pub mod prelude {
    pub use super::PlayerSpaceshipMarker;
    pub use super::SpaceshipControlMode;
    pub use super::SpaceshipInputMarker;
    pub use super::SpaceshipPlayerInputPlugin;
    pub use super::SpaceshipPlayerInputPluginSet;
    pub use super::SpaceshipRotationInputActiveMarker;
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpaceshipPlayerInputPluginSet;

pub struct SpaceshipPlayerInputPlugin;

impl Plugin for SpaceshipPlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpaceshipControlMode>();

        app.add_observer(insert_player_spaceship_controller);
        app.add_observer(insert_player_spaceship_freelook);
        app.add_observer(insert_player_spaceship_turret);

        app.add_systems(
            Update,
            (
                update_chase_camera_input.before(ChaseCameraPluginSet),
                update_controller_target_rotation_torque,
                update_turret_target_input,
                sync_spaceship_control_mode,
            )
                .in_set(SpaceshipPlayerInputPluginSet)
                .chain(),
        );
    }
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct PlayerSpaceshipMarker;

#[derive(Resource, Default, Clone, Debug)]
pub enum SpaceshipControlMode {
    #[default]
    Normal,
    FreeLook,
    Turret,
}

#[derive(Component, Debug, Clone)]
pub struct SpaceshipInputMarker;

#[derive(Component, Debug, Clone)]
pub struct SpaceshipControllerInputMarker;

#[derive(Component, Debug, Clone)]
pub struct SpaceshipFreeLookInputMarker;

#[derive(Component, Debug, Clone)]
pub struct SpaceshipTurretInputMarker;

#[derive(Component, Debug, Clone)]
pub struct SpaceshipRotationInputActiveMarker;

fn insert_player_spaceship_controller(
    add: On<Add, PlayerSpaceshipMarker>,
    mut commands: Commands,
    q_spaceship: Query<Entity, With<SpaceshipRootMarker>>,
) {
    let entity = add.entity;
    debug!(
        "PlayerSpaceshipMarker added to entity {:?}, inserting SpaceshipRotationInputMarker to it.",
        entity
    );

    let Ok(spaceship) = q_spaceship.get(entity) else {
        warn!(
            "Failed to get SpaceshipRootMarker for PlayerSpaceshipMarker entity {:?}",
            add.entity
        );
        return;
    };

    commands.entity(spaceship).with_children(|parent| {
        parent.spawn((
            SpaceshipInputMarker,
            SpaceshipControllerInputMarker,
            SpaceshipRotationInputActiveMarker,
            PointRotation::default(),
        ));
    });
}

fn insert_player_spaceship_freelook(
    add: On<Add, PlayerSpaceshipMarker>,
    mut commands: Commands,
    q_spaceship: Query<Entity, With<SpaceshipRootMarker>>,
) {
    let entity = add.entity;
    debug!(
        "PlayerSpaceshipMarker added to entity {:?}, inserting SpaceshipFreeLookInputMarker to it.",
        entity
    );

    let Ok(spaceship) = q_spaceship.get(entity) else {
        warn!(
            "Failed to get SpaceshipRootMarker for PlayerSpaceshipMarker entity {:?}",
            add.entity
        );
        return;
    };

    commands.entity(spaceship).with_children(|parent| {
        parent.spawn((
            SpaceshipInputMarker,
            SpaceshipFreeLookInputMarker,
            PointRotation::default(),
        ));
    });
}

fn insert_player_spaceship_turret(
    add: On<Add, PlayerSpaceshipMarker>,
    mut commands: Commands,
    q_spaceship: Query<Entity, With<SpaceshipRootMarker>>,
) {
    let entity = add.entity;
    debug!(
        "PlayerSpaceshipMarker added to entity {:?}, inserting SpaceshipTurretInputMarker to it.",
        entity
    );

    let Ok(spaceship) = q_spaceship.get(entity) else {
        warn!(
            "Failed to get SpaceshipRootMarker for PlayerSpaceshipMarker entity {:?}",
            add.entity
        );
        return;
    };

    commands.entity(spaceship).with_children(|parent| {
        parent.spawn((
            SpaceshipInputMarker,
            SpaceshipTurretInputMarker,
            PointRotation::default(),
        ));
    });
}

fn update_chase_camera_input(
    camera: Single<&mut ChaseCameraInput, With<ChaseCamera>>,
    spaceship: Single<&Transform, (With<SpaceshipRootMarker>, With<PlayerSpaceshipMarker>)>,
    point_rotation: Single<
        &PointRotationOutput,
        (With<SpaceshipInputMarker>, With<SpaceshipRotationInputActiveMarker>),
    >,
) {
    let mut camera_input = camera.into_inner();
    let spaceship_transform = spaceship.into_inner();
    let point_rotation = point_rotation.into_inner();

    camera_input.anchor_pos = spaceship_transform.translation;
    camera_input.achor_rot = **point_rotation;
}

fn update_controller_target_rotation_torque(
    point_rotation: Single<
        (&PointRotationOutput, &ChildOf),
        (With<SpaceshipInputMarker>, With<SpaceshipControllerInputMarker>),
    >,
    mut q_controller: Query<
        (&mut ControllerSectionRotationInput, &ChildOf),
        With<ControllerSectionMarker>,
    >,
) {
    let (point_rotation, ChildOf(parent)) = point_rotation.into_inner();

    for (mut controller, _) in q_controller
        .iter_mut()
        .filter(|(_, ChildOf(c_parent))| *c_parent == *parent)
    {
        **controller = **point_rotation;
    }
}

fn update_turret_target_input(
    point_rotation: Single<
        (&PointRotationOutput, &ChildOf),
        (With<SpaceshipInputMarker>, With<SpaceshipTurretInputMarker>),
    >,
    mut q_turret: Query<(&mut TurretSectionTargetInput, &ChildOf), With<TurretSectionMarker>>,
    q_spaceship: Query<&Transform, With<SpaceshipRootMarker>>,
) {
    let (point_rotation, ChildOf(parent)) = point_rotation.into_inner();
    let Ok(spaceship_transform) = q_spaceship.get(*parent) else {
        warn!("Turret's parent spaceship not found for TurretSectionMarker");
        return;
    };

    for (mut turret, _) in q_turret
        .iter_mut()
        .filter(|(_, ChildOf(t_parent))| *t_parent == *parent)
    {
        let forward = **point_rotation * -Vec3::Z;
        let position = spaceship_transform.translation;
        let distance = 100.0;

        **turret = Some(position + forward * distance);
    }
}

fn sync_spaceship_control_mode(
    mut commands: Commands,
    mode: Res<SpaceshipControlMode>,
    spaceship_input_rotation: Single<
        (Entity, &PointRotationOutput),
        With<SpaceshipControllerInputMarker>,
    >,
    spaceship_input_free_look: Single<Entity, With<SpaceshipFreeLookInputMarker>>,
    spaceship_input_turret: Single<Entity, With<SpaceshipTurretInputMarker>>,
    camera: Single<Entity, With<ChaseCamera>>,
) {
    if !mode.is_changed() {
        return;
    }

    let (spaceship_input_rotation, point_rotation) = spaceship_input_rotation.into_inner();
    let spaceship_input_free_look = spaceship_input_free_look.into_inner();
    let spaceship_input_combat = spaceship_input_turret.into_inner();
    let camera = camera.into_inner();

    match *mode {
        SpaceshipControlMode::Normal => {
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
        SpaceshipControlMode::FreeLook => {
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
        SpaceshipControlMode::Turret => {
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
