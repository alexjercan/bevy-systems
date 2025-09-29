//! In this example, I want to demo how to use StableTorquePdController to rotate a spaceship to
//! follow the mouse cursor. The spaceship will rotate to face the mouse cursor when moved.

use avian3d::{math::*, prelude::*};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_systems::prelude::*;
use clap::Parser;

#[derive(Parser)]
#[command(name = "spaceship_rotation")]
#[command(version = "0.1")]
#[command(about = "Example for the StableTorquePdController", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();

    let mut app = new_gui_app();

    app.add_plugins(PhysicsPlugins::default());
    app.insert_resource(Gravity::ZERO);
    app.add_systems(Startup, setup);

    app.add_plugins(EnhancedInputPlugin);
    app.add_input_context::<CameraInputMarker>();
    app.add_observer(update_camera_rotation_input);
    app.add_observer(update_camera_zoom_input);

    app.add_plugins(OrbitCameraPlugin);
    app.add_plugins(SphereRandomOrbitPlugin);
    app.add_plugins(StableTorquePdControllerPlugin);
    app.add_systems(Update, update_spaceship_target_rotation);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Setting up the scene...");

    // Spawn a 3D camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 20.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        OrbitCamera::default(),
        CameraInputMarker,
        actions!(
            CameraInputMarker[
                (
                    Action::<CameraInputRotate>::new(),
                    Bindings::spawn((
                        // Bevy requires single entities to be wrapped in `Spawn`.
                        // You can attach modifiers to individual bindings as well.
                        Spawn((Binding::mouse_motion(), Scale::splat(0.1), Negate::all())),
                        Axial::right_stick().with((Scale::splat(2.0), Negate::x())),
                    )),
                ),
                (
                    Action::<CameraInputZoom>::new(),
                    Scale::splat(1.0),
                    Bindings::spawn((
                        Spawn((Binding::mouse_wheel(), SwizzleAxis::YXZ)),
                        Bidirectional::up_down_dpad(),
                    ))
                ),
            ]
        ),
    ));

    // Spawn a light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -FRAC_PI_2, 0.0, 0.0)),
        GlobalTransform::default(),
    ));

    // Spawn a spaceship entity (a rectangle with some features to figure out its orientation)
    commands.spawn((
        Name::new("Spaceship"),
        RigidBody::Dynamic,
        Collider::cylinder(0.5, 1.0),
        ColliderDensity(2.0),
        StableTorquePdController {
            frequency: 2.0,
            damping_ratio: 1.0,
            max_torque: 10.0,
        },
        Transform::default(),
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
            )
        ],
    ));

    // Spawn a target entity to visualize the target rotation
    commands.spawn((
        Name::new("Spaceship Rotation Target"),
        SpaceshipRotationTargetMarker,
        SphereOrbit {
            radius: 5.0,
            angular_speed: 5.0,
            center: Vec3::ZERO,
        },
        StableTorquePdControllerTarget(Quat::IDENTITY),
        Transform::from_xyz(0.0, 0.0, -5.0),
        Visibility::Visible,
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.2))),
    ));
}

#[derive(Component, Debug, Clone)]
struct CameraInputMarker;

#[derive(InputAction)]
#[action_output(Vec2)]
struct CameraInputRotate;

#[derive(InputAction)]
#[action_output(f32)]
struct CameraInputZoom;

fn update_camera_rotation_input(
    trigger: Trigger<Fired<CameraInputRotate>>,
    mut q_input: Query<&mut OrbitCameraInput, With<CameraInputMarker>>,
) {
    if let Ok(mut input) = q_input.get_mut(trigger.target()) {
        input.orbit = trigger.value;
    }
}

fn update_camera_zoom_input(
    trigger: Trigger<Fired<CameraInputZoom>>,
    mut q_input: Query<&mut OrbitCameraInput, With<CameraInputMarker>>,
) {
    if let Ok(mut input) = q_input.get_mut(trigger.target()) {
        input.zoom = trigger.value;
    }
}

#[derive(Component, Debug, Clone)]
struct SpaceshipRotationTargetMarker;

fn update_spaceship_target_rotation(
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
