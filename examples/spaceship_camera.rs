//! In this example, I want to demo how to use StableTorquePdController to rotate a spaceship to
//! follow the mouse cursor. The spaceship will rotate to face the mouse cursor when moved.

mod helpers;

use avian3d::{math::*, prelude::*};
use bevy::prelude::*;
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
    app.add_plugins(PrettyScenePlugin);

    // We need to enable the physics plugins to have access to RigidBody and other components.
    // We will also disable gravity for this example, since we are in space, duh.
    app.add_plugins(PhysicsPlugins::default());
    app.insert_resource(Gravity::ZERO);

    // Setup the scene with some entities, to have something to look at.
    app.add_systems(OnEnter(GameStates::Playing), setup);

    // Setup the input system to get input from the mouse and keyboard.
    app.add_plugins(EnhancedInputPlugin);
    app.add_input_context::<PlayerInputMarker>();
    app.add_observer(on_rotation_input);
    app.add_observer(on_rotation_input_completed);
    app.add_observer(on_thruster_input);
    app.add_observer(on_thruster_input_completed);
    app.add_observer(on_control_mode_input_started);
    app.add_observer(on_control_mode_input_completed);

    app.insert_resource(SpaceshipControlMode::default());

    app.add_plugins(ChaseCameraPlugin);
    app.add_plugins(PointRotationPlugin);
    app.add_plugins(StableTorquePdControllerPlugin);
    app.add_systems(
        Update,
        (
            update_spaceship_target_rotation_torque,
            update_chase_camera_input,
            sync_spaceship_control_mode,
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
}

#[derive(Component, Debug, Clone)]
struct SpaceshipRotationInputActiveMarker;

fn sync_spaceship_control_mode(
    mut commands: Commands,
    mode: Res<SpaceshipControlMode>,
    spaceship_input_rotation: Single<(Entity, &PointRotationOutput), With<SpaceshipRotationInputMarker>>,
    spaceship_input_free_look: Single<Entity, With<FreeLookRotationInputMarker>>,
    camera: Single<Entity, With<ChaseCamera>>,
) {
    if !mode.is_changed() {
        return;
    }

    let (spaceship_input_rotation, point_rotation) = spaceship_input_rotation.into_inner();
    let spaceship_input_free_look = spaceship_input_free_look.into_inner();
    let camera = camera.into_inner();

    match *mode {
        SpaceshipControlMode::Normal => {
            commands
                .entity(spaceship_input_rotation)
                .insert(SpaceshipRotationInputActiveMarker);
            commands
                .entity(spaceship_input_free_look)
                .remove::<SpaceshipRotationInputActiveMarker>();
            commands.entity(camera).insert(
                ChaseCamera {
                    offset: Vec3::new(0.0, 5.0, -20.0),
                    focus_offset: Vec3::new(0.0, 0.0, 20.0),
                },
            );
        }
        SpaceshipControlMode::FreeLook => {
            commands
                .entity(spaceship_input_rotation)
                .remove::<SpaceshipRotationInputActiveMarker>();
            commands
                .entity(spaceship_input_free_look)
                .insert(PointRotation { initial_rotation: **point_rotation })
                .insert(SpaceshipRotationInputActiveMarker);
            commands.entity(camera).insert(
                ChaseCamera {
                    offset: Vec3::new(0.0, 10.0, -30.0),
                    focus_offset: Vec3::new(0.0, 0.0, 0.0),
                },
            );
        }
    }
}

fn update_chase_camera_input(
    camera: Single<&mut ChaseCameraInput, With<ChaseCamera>>,
    spaceship: Single<&GlobalTransform, With<SpaceshipMarker>>,
    point_rotation: Single<&PointRotationOutput, With<SpaceshipRotationInputActiveMarker>>
) {
    let mut camera_input = camera.into_inner();
    let spaceship_transform = spaceship.into_inner();
    let rotation = point_rotation.into_inner();

    camera_input.anchor_pos = spaceship_transform.translation();
    camera_input.achor_rot = **rotation;
}

const FREQUENCY: f32 = 2.0;
const DAMPING_RATIO: f32 = 2.0;
const MAX_TORQUE: f32 = 1.0;

fn setup(
    mut commands: Commands,
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
                    bindings![MouseButton::Right, GamepadButton::LeftTrigger],
                ),
            ]
        ),
    ));

    // Spawn a RotationInput to consume the mouse movement and will be used to rotate the spaceship
    // TODO: Implement the consume only when in Normal mode
    commands.spawn((
        Name::new("Spaceship Rotation Input"),
        SpaceshipRotationInputMarker,
        SpaceshipRotationInputActiveMarker,
        PointRotation::default(),
    ));

    // Spawn a RotationInput to consume the mouse movement and will be used to rotate the free look
    // TODO: Implement the consume only when in FreeLook mode
    commands.spawn((
        Name::new("FreeLook Rotation Input"),
        FreeLookRotationInputMarker,
        PointRotation::default(),
    ));

    // Spawn a spaceship entity (a rectangle with some features to figure out its orientation)
    commands.spawn((
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
        children![
            (
                Name::new("Spaceship Renderer"),
                Mesh3d(meshes.add(Cylinder::new(0.5, 1.0))),
                MeshMaterial3d(materials.add(Color::srgb(0.2, 0.7, 0.9))),
                Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
            ),
            (
                Name::new("Spaceship Thruster"),
                Mesh3d(meshes.add(Cone::new(0.5, 0.5))),
                MeshMaterial3d(materials.add(Color::srgb(0.9, 0.3, 0.2))),
                Transform::from_xyz(0.0, 0.0, 0.5).with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
            ),
            (
                Name::new("Spaceship Window"),
                Mesh3d(meshes.add(Cylinder::new(0.2, 0.1))),
                MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 1.0))),
                Transform::from_xyz(0.0, 0.5, -0.1),
            )
        ],
        // Debug stuff
        DebugAxisMarker,
    ));

    // Spawn a 3D camera with a chase camera component
    // TODO: Implement this thing
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
    ));
}

#[derive(Component, Debug, Clone)]
struct SpaceshipMarker;

#[derive(Component, Debug, Clone)]
struct PlayerInputMarker;

#[derive(Component, Debug, Clone)]
struct SpaceshipRotationInputMarker;

#[derive(Component, Debug, Clone)]
struct FreeLookRotationInputMarker;

#[derive(InputAction)]
#[action_output(Vec2)]
struct CameraInputRotate;

#[derive(InputAction)]
#[action_output(bool)]
struct ThrusterInput;

#[derive(InputAction)]
#[action_output(bool)]
struct FreeLookInput;

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

fn on_thruster_input(_: Trigger<Fired<ThrusterInput>>, spaceship: Single<(&mut ExternalImpulse, &GlobalTransform), With<SpaceshipMarker>>) {
    let (mut force, spaceship_transform) = spaceship.into_inner();

    let thrust_direction = spaceship_transform.forward();
    let thrust_magnitude = 1.0;

    force.apply_impulse(thrust_direction * thrust_magnitude);
}

fn on_thruster_input_completed(_: Trigger<Completed<ThrusterInput>>) {
}

fn on_control_mode_input_started(_: Trigger<Started<FreeLookInput>>, mut mode: ResMut<SpaceshipControlMode>) {
    *mode = SpaceshipControlMode::FreeLook;
}

fn on_control_mode_input_completed(
    _: Trigger<Completed<FreeLookInput>>,
    mut mode: ResMut<SpaceshipControlMode>,
) {
    *mode = SpaceshipControlMode::Normal;
}
