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
#[command(name = "spaceship_torque")]
#[command(version = "0.1")]
#[command(about = "Example for the StableTorquePdController", long_about = None)]
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
    // For a WASD camera, see the `wasd_camera` plugin.
    app.add_plugins(WASDCameraPlugin);
    app.add_plugins(EnhancedInputPlugin);
    app.add_input_context::<CameraInputMarker>();
    app.add_observer(update_camera_rotation_input);
    app.add_observer(update_camera_rotation_input_completed);
    app.add_observer(update_camera_move_input);
    app.add_observer(update_camera_move_input_completed);
    app.add_observer(update_camera_elevation_input);
    app.add_observer(update_camera_elevation_input_completed);

    // Add some util systems to handle random orbiting and drawing gizmos.
    app.add_plugins(SphereRandomOrbitPlugin);
    app.add_systems(Update, draw_debug_gizmos);

    // Add the torque controller plugin
    app.add_plugins(StableTorquePdControllerPlugin);
    app.add_systems(Update, update_spaceship_target_rotation_torque);

    app.run();
}

#[derive(Component, Debug, Clone)]
struct SpaceshipMarker;

#[derive(Component, Debug, Clone)]
struct SpaceshipRotationTargetMarker;

fn update_spaceship_target_rotation_torque(
    target: Single<&Transform, With<SpaceshipRotationTargetMarker>>,
    controller: Single<
        (&mut StableTorquePdControllerTarget, &Transform),
        With<StableTorquePdController>,
    >,
) {
    let target_transform = target.into_inner();
    let (mut controller_target, controller_transform) = controller.into_inner();

    let direction =
        (target_transform.translation - controller_transform.translation).normalize_or_zero();
    let forward = controller_transform.forward();
    let angle = forward.angle_between(direction);
    let axis = forward.cross(direction).normalize_or_zero();

    let target_rotation = Quat::from_axis_angle(axis, angle) * controller_transform.rotation;

    **controller_target = target_rotation;
}

const FREQUENCY: f32 = 2.0;
const DAMPING_RATIO: f32 = 1.0;
const MAX_TORQUE: f32 = 1.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>,
) {
    info!("Setting up the scene...");

    // Spawn a 3D camera
    commands.spawn((
        Name::new("3D Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 20.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        // WASD Camera Controller for moving around the scene
        WASDCamera::default(),
        // Input Actions for controlling the camera
        CameraInputMarker,
        actions!(
            CameraInputMarker[
                (
                    Action::<CameraInputRotate>::new(),
                    Bindings::spawn((
                        // Bevy requires single entities to be wrapped in `Spawn`.
                        // You can attach modifiers to individual bindings as well.
                        Spawn((Binding::mouse_motion(), Scale::splat(0.01), Negate::none())),
                        Axial::right_stick().with((Scale::splat(2.0), Negate::x())),
                    )),
                ),
                (
                    Action::<CameraInputMove>::new(),
                    Scale::splat(1.0),
                    Bindings::spawn((
                        Cardinal::wasd_keys(),
                        Axial::left_stick().with(Negate::y()),
                    ))
                ),
                (
                    Action::<CameraInputElevation>::new(),
                    Scale::splat(1.0),
                    Bindings::spawn(
                        Bidirectional::<Binding, Binding> {
                            positive: KeyCode::Space.into(),
                            negative: KeyCode::ShiftLeft.into(),
                        }
                    ),
                ),
            ]
        ),
        Skybox {
            image: game_assets.cubemap.clone(),
            brightness: 1000.0,
            ..default()
        },
    ));

    // Spawn a spaceship entity (a rectangle with some features to figure out its orientation)
    commands.spawn((
        Name::new("Spaceship"),
        SpaceshipMarker,
        // Physics components
        RigidBody::Dynamic,
        Collider::cylinder(0.5, 1.0),
        ColliderDensity(2.0),
        StableTorquePdController {
            frequency: FREQUENCY,
            damping_ratio: DAMPING_RATIO,
            max_torque: MAX_TORQUE,
        },
        Transform::default(),
        GlobalTransform::default(),
        // Rendering components
        Visibility::Visible,
        spaceship_render(&mut meshes, &mut materials),
        // Debug stuff
        DebugAxisMarker,
    ));

    // Spawn a target entity to visualize the target rotation
    commands.spawn((
        Name::new("Spaceship Rotation Target"),
        SpaceshipRotationTargetMarker,
        RandomSphereOrbit {
            radius: 10.0,
            angular_speed: 5.0,
            center: Vec3::ZERO,
        },
        Transform::from_xyz(0.0, 0.0, -5.0),
        Visibility::Visible,
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.2))),
        DebugAxisMarker,
    ));
}

#[derive(Component, Debug, Clone)]
struct CameraInputMarker;

#[derive(InputAction)]
#[action_output(Vec2)]
struct CameraInputRotate;

#[derive(InputAction)]
#[action_output(Vec2)]
struct CameraInputMove;

#[derive(InputAction)]
#[action_output(f32)]
struct CameraInputElevation;

fn update_camera_rotation_input(
    trigger: Trigger<Fired<CameraInputRotate>>,
    mut q_input: Query<&mut WASDCameraInput, With<CameraInputMarker>>,
) {
    if let Ok(mut input) = q_input.get_mut(trigger.target()) {
        input.pan = trigger.value;
    }
}

fn update_camera_rotation_input_completed(
    trigger: Trigger<Completed<CameraInputRotate>>,
    mut q_input: Query<&mut WASDCameraInput, With<CameraInputMarker>>,
) {
    if let Ok(mut input) = q_input.get_mut(trigger.target()) {
        input.pan = Vec2::ZERO;
    }
}

fn update_camera_move_input(
    trigger: Trigger<Fired<CameraInputMove>>,
    mut q_input: Query<&mut WASDCameraInput, With<CameraInputMarker>>,
) {
    if let Ok(mut input) = q_input.get_mut(trigger.target()) {
        input.wasd = trigger.value;
    }
}

fn update_camera_move_input_completed(
    trigger: Trigger<Completed<CameraInputMove>>,
    mut q_input: Query<&mut WASDCameraInput, With<CameraInputMarker>>,
) {
    if let Ok(mut input) = q_input.get_mut(trigger.target()) {
        input.wasd = Vec2::ZERO;
    }
}

fn update_camera_elevation_input(
    trigger: Trigger<Fired<CameraInputElevation>>,
    mut q_input: Query<&mut WASDCameraInput, With<CameraInputMarker>>,
) {
    if let Ok(mut input) = q_input.get_mut(trigger.target()) {
        input.vertical = trigger.value;
    }
}

fn update_camera_elevation_input_completed(
    trigger: Trigger<Completed<CameraInputElevation>>,
    mut q_input: Query<&mut WASDCameraInput, With<CameraInputMarker>>,
) {
    if let Ok(mut input) = q_input.get_mut(trigger.target()) {
        input.vertical = 0.0;
    }
}

fn draw_debug_gizmos(
    q_spaceship: Query<(&Transform, &StableTorquePdControllerTarget), With<SpaceshipMarker>>,
    mut gizmos: Gizmos,
) {
    for (transform, target) in &q_spaceship {
        let origin = transform.translation;
        let desired_forward = target.mul_vec3(-Vec3::Z).normalize_or_zero();
        let target_point = origin + desired_forward * 10.0;

        gizmos.line(origin, target_point, Color::srgb(1.0, 1.0, 0.0));
    }
}
