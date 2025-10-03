//! In this example, I want to demo how to use StableTorquePdController to rotate a spaceship to
//! follow the mouse cursor. The spaceship will rotate to face the mouse cursor when moved.

mod helpers;

use avian3d::prelude::*;
use bevy::{core_pipeline::Skybox, prelude::*};
use bevy_enhanced_input::prelude::*;
use bevy_systems::prelude::*;
use clap::Parser;
use helpers::prelude::*;

#[derive(Parser)]
#[command(name = "spaceship_camera")]
#[command(version = "0.1")]
#[command(about = "Example for the 3rd person camera controller", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();

    let mut app = new_gui_app();
    app.add_plugins(GameAssetsPlugin);
    app.add_plugins(DebugGizmosPlugin);

    // We need to enable the physics plugins to have access to RigidBody and other components.
    // We will also disable gravity for this example, since we are in space, duh.
    app.add_plugins(PhysicsPlugins::default());
    app.add_plugins(PhysicsDebugPlugin::default());
    app.insert_resource(Gravity::ZERO);

    // Setup the scene with some entities, to have something to look at.
    app.add_systems(OnEnter(GameStates::Playing), (setup, setup_simple_scene));

    // Setup the input system to get input from the mouse and keyboard.
    app.add_plugins(EnhancedInputPlugin);
    app.add_input_context::<PlayerInputMarker>();
    app.add_observer(on_rotation_input);
    app.add_observer(on_rotation_input_completed);
    app.add_observer(on_thruster_input);
    app.add_observer(on_thruster_input_completed);
    app.add_observer(on_free_mode_input_started);
    app.add_observer(on_free_mode_input_completed);
    app.add_observer(on_combat_input_started);
    app.add_observer(on_combat_input_completed);

    app.insert_resource(SpaceshipControlMode::default());

    // Chase Camera Plugin to have a 3rd person camera following the spaceship
    app.add_plugins(ChaseCameraPlugin);
    // Point Rotation Plugin to convert mouse movement to a target rotation
    app.add_plugins(PointRotationPlugin);
    // Torque Controller Plugin to rotate the spaceship to face the target rotation
    app.add_plugins(StableTorquePdControllerPlugin);
    // for debug to have a random orbiting object
    app.add_plugins(SphereRandomOrbitPlugin);
    // Rotation Plugin for the turret facing direction
    app.add_plugins(SmoothLookRotationPlugin);
    app.add_plugins(TurretPlugin);
    app.add_systems(
        Update,
        (
            update_spaceship_target_rotation_torque,
            update_chase_camera_input.before(ChaseCameraPluginSet),
            sync_spaceship_control_mode,
            update_turret_target_input,
        ),
    );

    app.run();
}

fn update_spaceship_target_rotation_torque(
    point_rotation: Single<&PointRotationOutput, With<SpaceshipRotationInputMarker>>,
    controller: Single<&mut StableTorquePdControllerTarget, With<StableTorquePdController>>,
) {
    let rotation = point_rotation.into_inner();
    let mut controller_target = controller.into_inner();
    **controller_target = **rotation;
}

#[derive(Resource, Default, Clone, Debug)]
enum SpaceshipControlMode {
    #[default]
    Normal,
    FreeLook,
    Combat,
}

#[derive(Component, Debug, Clone)]
struct SpaceshipRotationInputActiveMarker;

fn sync_spaceship_control_mode(
    mut commands: Commands,
    mode: Res<SpaceshipControlMode>,
    spaceship_input_rotation: Single<
        (Entity, &PointRotationOutput),
        With<SpaceshipRotationInputMarker>,
    >,
    spaceship_input_free_look: Single<Entity, With<FreeLookRotationInputMarker>>,
    spaceship_input_combat: Single<Entity, With<CombatRotationInputMarker>>,
    camera: Single<Entity, With<ChaseCamera>>,
) {
    if !mode.is_changed() {
        return;
    }

    let (spaceship_input_rotation, point_rotation) = spaceship_input_rotation.into_inner();
    let spaceship_input_free_look = spaceship_input_free_look.into_inner();
    let spaceship_input_combat = spaceship_input_combat.into_inner();
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
            });
        }
        SpaceshipControlMode::Combat => {
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
                focus_offset: Vec3::new(0.0, 0.0, 0.0),
            });
        }
    }
}

fn update_chase_camera_input(
    camera: Single<&mut ChaseCameraInput, With<ChaseCamera>>,
    spaceship: Single<&GlobalTransform, With<SpaceshipMarker>>,
    point_rotation: Single<&PointRotationOutput, With<SpaceshipRotationInputActiveMarker>>,
) {
    let mut camera_input = camera.into_inner();
    let spaceship_transform = spaceship.into_inner();
    let rotation = point_rotation.into_inner();

    camera_input.anchor_pos = spaceship_transform.translation();
    camera_input.achor_rot = **rotation;
}

fn update_turret_target_input(
    target: Single<&GlobalTransform, With<PDCTurretTargetMarker>>,
    turret: Single<&mut TurretTargetInput, With<PDCTurretMarker>>,
    mode: Res<SpaceshipControlMode>,
    point_rotation: Single<&PointRotationOutput, With<CombatRotationInputMarker>>,
    spaceship: Single<&GlobalTransform, With<SpaceshipMarker>>,
) {
    if matches!(*mode, SpaceshipControlMode::Combat) {
        let rotation = point_rotation.into_inner();
        let mut turret_target = turret.into_inner();
        let spaceship_transform = spaceship.into_inner();

        let forward = **rotation * -Vec3::Z;
        let position = spaceship_transform.translation();
        let distance = 100.0;

        **turret_target = position + forward * distance;
    } else {
        let target_transform = target.into_inner();
        let mut turret_target = turret.into_inner();

        **turret_target = target_transform.translation();
    }
}

const FREQUENCY: f32 = 2.0;
const DAMPING_RATIO: f32 = 2.0;
const MAX_TORQUE: f32 = 1.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>,
) {
    // Spawn a player input controller entity to hold the input from the player
    commands.spawn((
        Name::new("Player Input Controller"),
        Transform::default(),
        GlobalTransform::default(),
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
                    Action::<ThrusterInput>::new(),
                    bindings![KeyCode::KeyW, GamepadButton::RightTrigger],
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

    // Spawn a RotationInput to consume the mouse movement and will be used to rotate the spaceship
    commands.spawn((
        Name::new("Spaceship Rotation Input"),
        SpaceshipRotationInputMarker,
        SpaceshipRotationInputActiveMarker,
        PointRotation::default(),
    ));

    // Spawn a RotationInput to consume the mouse movement and will be used to rotate the free look
    commands.spawn((
        Name::new("FreeLook Rotation Input"),
        FreeLookRotationInputMarker,
        PointRotation::default(),
    ));

    // Spawn a RotationInput to consume the mouse movement and will be used to rotate the combat
    commands.spawn((
        Name::new("Combat Rotation Input"),
        CombatRotationInputMarker,
        PointRotation::default(),
    ));

    // Spawn a spaceship entity (a rectangle with some features to figure out its orientation)
    let spaceship = commands
        .spawn((
            Name::new("Spaceship"),
            SpaceshipMarker,
            RigidBody::Dynamic,
            Collider::cylinder(0.5, 1.0),
            ColliderDensity(2.0),
            // PD Controller to rotate the spaceship to face the target rotation
            StableTorquePdController {
                frequency: FREQUENCY,
                damping_ratio: DAMPING_RATIO,
                max_torque: MAX_TORQUE,
            },
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Visible,
            spaceship_render(&mut meshes, &mut materials),
        ))
        .id();

    let turret = commands
        .spawn((
            Name::new("Turret Anchor"),
            RigidBody::Dynamic,
            Collider::sphere(0.4),
            ColliderDensity(0.01),
            Position::from_xyz(0.8, 0.0, 0.0),
            Visibility::Visible,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("Turret"),
                    PDCTurretMarker,
                    TurretBaseMarker,
                    TurretTargetInput(Vec3::ZERO),
                    Transform::from_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
                    GlobalTransform::default(),
                    Visibility::Inherited,
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Name::new("Turret Rotator"),
                            TurretRotatorMarker,
                            SmoothLookRotation {
                                initial_yaw: 0.0,
                                initial_pitch: 0.0,
                                yaw_speed: std::f32::consts::PI,   // 180 degrees per second
                                pitch_speed: std::f32::consts::PI, // 180 degrees per second
                                min_pitch: Some(-std::f32::consts::FRAC_PI_6),
                                max_pitch: None,
                            },
                            Transform::from_xyz(0.0, 0.0, 0.0),
                            GlobalTransform::default(),
                            Visibility::Inherited,
                            // See the turret facing direction
                            DebugAxisMarker,
                        ))
                        .with_child((
                            Name::new("Render"),
                            Transform::from_scale(Vec3::splat(0.5)),
                            Visibility::Inherited,
                            turret_render(&mut meshes, &mut materials),
                        ));
                });
        })
        .id();

    commands
        .spawn(FixedJoint::new(spaceship, turret).with_local_anchor_1(Vec3::new(0.8, 0.0, 0.0)));

    // Spawn a 3D camera with a chase camera component
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        ChaseCamera::default(),
        Visibility::Visible,
        Skybox {
            image: game_assets.cubemap.clone(),
            brightness: 1000.0,
            ..default()
        },
    ));

    // Spawn a target entity to visualize the target rotation
    commands.spawn((
        Name::new("Turret Target"),
        PDCTurretTargetMarker,
        // RandomSphereOrbit {
        //     radius: 5.0,
        //     angular_speed: 5.0,
        //     center: Vec3::ZERO,
        // },
        Transform::from_xyz(0.0, 0.0, -5.0),
        Visibility::Visible,
        Mesh3d(meshes.add(Cuboid::new(3.0, 3.0, 3.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.2))),
    ));
}

#[derive(Component, Debug, Clone)]
struct SpaceshipMarker;

#[derive(Component, Clone, Copy, Debug, Reflect)]
struct PDCTurretMarker;

#[derive(Component, Clone, Copy, Debug, Reflect)]
struct PDCTurretTargetMarker;

#[derive(Component, Debug, Clone)]
struct PlayerInputMarker;

#[derive(Component, Debug, Clone)]
struct SpaceshipRotationInputMarker;

#[derive(Component, Debug, Clone)]
struct FreeLookRotationInputMarker;

#[derive(Component, Debug, Clone)]
struct CombatRotationInputMarker;

#[derive(InputAction)]
#[action_output(Vec2)]
struct CameraInputRotate;

#[derive(InputAction)]
#[action_output(bool)]
struct ThrusterInput;

#[derive(InputAction)]
#[action_output(bool)]
struct FreeLookInput;

#[derive(InputAction)]
#[action_output(bool)]
struct CombatInput;

fn on_rotation_input(
    trigger: Trigger<Fired<CameraInputRotate>>,
    mut q_input: Query<&mut PointRotationInput, With<SpaceshipRotationInputActiveMarker>>,
) {
    for mut input in &mut q_input {
        **input = trigger.value;
    }
}

fn on_rotation_input_completed(
    _: Trigger<Completed<CameraInputRotate>>,
    mut q_input: Query<&mut PointRotationInput>,
) {
    for mut input in &mut q_input {
        **input = Vec2::ZERO;
    }
}

fn on_thruster_input(
    _: Trigger<Fired<ThrusterInput>>,
    spaceship: Single<(&mut ExternalImpulse, &GlobalTransform), With<SpaceshipMarker>>,
) {
    let (mut force, spaceship_transform) = spaceship.into_inner();

    let thrust_direction = spaceship_transform.forward();
    let thrust_magnitude = 1.0;

    force.apply_impulse(thrust_direction * thrust_magnitude);
}

fn on_thruster_input_completed(_: Trigger<Completed<ThrusterInput>>) {}

fn on_free_mode_input_started(
    _: Trigger<Started<FreeLookInput>>,
    mut mode: ResMut<SpaceshipControlMode>,
) {
    *mode = SpaceshipControlMode::FreeLook;
}

fn on_free_mode_input_completed(
    _: Trigger<Completed<FreeLookInput>>,
    mut mode: ResMut<SpaceshipControlMode>,
) {
    *mode = SpaceshipControlMode::Normal;
}

fn on_combat_input_started(
    _: Trigger<Started<CombatInput>>,
    mut mode: ResMut<SpaceshipControlMode>,
) {
    *mode = SpaceshipControlMode::Combat;
}

fn on_combat_input_completed(
    _: Trigger<Completed<CombatInput>>,
    mut mode: ResMut<SpaceshipControlMode>,
) {
    *mode = SpaceshipControlMode::Normal;
}

/// This will be root component for the turret entity. It will hold as a child the rotation part
/// and it will provide a reference frame for the rotation.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct TurretBaseMarker;

/// This will be the component for the rotating part of the turret. It will be a child of the base.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct TurretRotatorMarker;

/// This will be the turret's target component input. It will be a Vec3 target position that we
/// want to aim at in world space.
#[derive(Component, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct TurretTargetInput(pub Vec3);

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TurretBaseMarker>()
            .register_type::<TurretRotatorMarker>()
            .register_type::<TurretTargetInput>();

        app.add_systems(
            Update,
            (
                update_turret_target_system,
                sync_turret_transform_system,
            )
        );
    }
}

fn update_turret_target_system(
    q_turret: Query<(&GlobalTransform, Option<&TurretTargetInput>), With<TurretBaseMarker>>,
    mut q_rotator: Query<(&mut SmoothLookRotationTarget, &ChildOf), With<TurretRotatorMarker>>,
) {
    for (mut rotator_target, &ChildOf(parent)) in &mut q_rotator {
        let Ok((turret_transform, target_input)) = q_turret.get(parent) else {
            warn!("TurretRotatorMarker's parent is not a TurretBaseMarker");
            continue;
        };

        let Some(target_input) = target_input else {
            println!("No target input for turret");
            continue;
        };

        let world_to_turret = turret_transform.compute_matrix().inverse();
        let world_pos = **target_input;
        let local_pos = world_to_turret.transform_point3(world_pos);

        let dir_local = local_pos.normalize_or_zero();

        let (yaw, pitch, _) = Quat::from_rotation_arc(Vec3::NEG_Z, dir_local).to_euler(EulerRot::YXZ);

        rotator_target.yaw = yaw;
        rotator_target.pitch = pitch;
    }
}

fn sync_turret_transform_system(
    mut q_rotator: Query<(&SmoothLookRotationOutput, &mut Transform), With<TurretRotatorMarker>>,
) {
    for (output, mut transform) in &mut q_rotator {
        *transform = Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, output.yaw, output.pitch, 0.0));
    }
}
