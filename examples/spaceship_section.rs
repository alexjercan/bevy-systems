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
#[command(name = "spaceship_section")]
#[command(version = "0.1")]
#[command(about = "Example for the section module", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();

    let mut app = new_gui_app();
    app.add_plugins(GameAssetsPlugin);
    app.add_plugins(DebugGizmosPlugin);

    // We need to enable the physics plugins to have access to RigidBody and other components.
    // We will also disable gravity for this example, since we are in space, duh.
    app.add_plugins(PhysicsPlugins::default());
    #[cfg(feature = "dev")]
    app.add_plugins(PhysicsDebugPlugin::default());
    app.insert_resource(Gravity::ZERO);

    // Setup the scene with some entities, to have something to look at.
    app.add_systems(
        OnEnter(GameStates::Playing),
        (setup, setup_spaceship, setup_simple_scene),
    );

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
    // for debug to have a random orbiting object
    app.add_plugins(SphereRandomOrbitPlugin);
    // Rotation Plugin for the turret facing direction
    app.add_plugins(SmoothLookRotationPlugin);

    // Add sections plugins
    app.add_plugins(SpaceshipPlugin);

    app.add_systems(
        Update,
        (
            update_chase_camera_input.before(ChaseCameraPluginSet),
            sync_spaceship_control_mode,
        ),
    );

    app.add_systems(
        Update,
        (
            update_spaceship_target_rotation_torque,
            update_turret_target_input,
        )
            .before(SpaceshipPluginSet),
    );

    app.run();
}

fn setup_spaceship(mut commands: Commands) {
    commands.spawn((
        spaceship_root(SpaceshipConfig { ..default() }),
        children![
            (controller_section(ControllerSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                frequency: 4.0,
                damping_ratio: 4.0,
                max_torque: 10.0,
                ..default()
            }),),
            (hull_section(HullSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                ..default()
            }),),
            (hull_section(HullSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..default()
            }),),
            (engine_section(EngineSectionConfig {
                thrust_magnitude: 1.0,
                transform: Transform::from_xyz(0.0, 0.0, 2.0),
                ..default()
            }),),
            (turret_section(TurretSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, -2.0)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                ..default()
            }),),
            // (turret_section(TurretSectionConfig {
            //     transform: Transform::from_xyz(0.0, 1.0, -1.0),
            //     ..default()
            // }),),
            // (turret_section(TurretSectionConfig {
            //     transform: Transform::from_xyz(0.0, -1.0, -1.0)
            //         .with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
            //     ..default()
            // }),),
            // (turret_section(TurretSectionConfig {
            //     transform: Transform::from_xyz(-1.0, 0.0, -1.0)
            //         .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            //     ..default()
            // }),),
            // (turret_section(TurretSectionConfig {
            //     transform: Transform::from_xyz(1.0, 0.0, -1.0)
            //         .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
            //     ..default()
            // }),),
        ],
    ));
}

fn update_spaceship_target_rotation_torque(
    point_rotation: Single<&PointRotationOutput, With<SpaceshipRotationInputMarker>>,
    controller: Single<&mut ControllerSectionRotationInput, With<ControllerSectionMarker>>,
) {
    let rotation = point_rotation.into_inner();
    let mut controller_target = controller.into_inner();
    **controller_target = **rotation;
}

fn update_chase_camera_input(
    camera: Single<&mut ChaseCameraInput, With<ChaseCamera>>,
    spaceship: Single<&GlobalTransform, With<SpaceshipRootMarker>>,
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
    mut q_turret: Query<&mut TurretSectionTargetInput, With<TurretSectionMarker>>,
    mode: Res<SpaceshipControlMode>,
    point_rotation: Single<&PointRotationOutput, With<CombatRotationInputMarker>>,
    spaceship: Single<&GlobalTransform, With<SpaceshipRootMarker>>,
) {
    if matches!(*mode, SpaceshipControlMode::Combat) {
        let rotation = point_rotation.into_inner();
        let spaceship_transform = spaceship.into_inner();

        for mut turret in &mut q_turret {
            let forward = **rotation * -Vec3::Z;
            let position = spaceship_transform.translation();
            let distance = 100.0;

            **turret = Some(position + forward * distance);
        }
    } else {
        let target_transform = target.into_inner();

        for mut turret in &mut q_turret {
            **turret = Some(target_transform.translation());
        }
    }
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
                focus_offset: Vec3::new(0.0, 0.0, 50.0),
            });
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
struct PDCTurretTargetMarker;

fn setup(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
        Transform::from_xyz(0.0, 0.0, -500.0),
        Visibility::Visible,
        Mesh3d(meshes.add(Cuboid::new(3.0, 3.0, 3.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.2))),
        Collider::cuboid(3.0, 3.0, 3.0),
        RigidBody::Static,
    ));
}

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

fn on_thruster_input(_: Trigger<Fired<ThrusterInput>>, mut q_input: Query<&mut EngineThrustInput>) {
    for mut input in &mut q_input {
        **input = 1.0;
    }
}

fn on_thruster_input_completed(
    _: Trigger<Completed<ThrusterInput>>,
    mut q_input: Query<&mut EngineThrustInput>,
) {
    for mut input in &mut q_input {
        **input = 0.0;
    }
}

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
