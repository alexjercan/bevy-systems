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
    app.add_observer(update_camera_rotation_input_enabled);
    app.add_observer(update_camera_rotation_input);
    app.add_observer(update_camera_zoom_input);
    app.add_observer(update_control_mode);

    app.insert_resource(ControlMode::default());

    app.add_plugins(OrbitCameraPlugin);
    app.add_plugins(SmoothZoomOrbitPlugin);
    app.add_plugins(SphereRandomOrbitPlugin);
    app.add_plugins(SmoothTargetRotationPlugin);
    app.add_plugins(StableTorquePdControllerPlugin);
    app.add_systems(
        Update,
        (
            update_spaceship_on_control_mode_changed,
            update_spaceship_target_rotation_torque,
            update_spaceship_target_rotation_smooth,
            draw_gizmos_from_spaceship_forward,
        ),
    );

    app.run();
}

const FREQUENCY: f32 = 2.0;
const DAMPING_RATIO: f32 = 1.0;
const MAX_TORQUE: f32 = 1.0;
const TURN_SPEED: f32 = std::f32::consts::PI; // 180 degrees per second

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
        SmoothZoomOrbit::default(),
        // Input Actions for controlling the camera
        CameraInputMarker,
        CameraInputState {
            rotate_enabled: false,
        },
        actions!(
            CameraInputMarker[
                (
                    Action::<CameraInputRotate>::new(),
                    Bindings::spawn((
                        // Bevy requires single entities to be wrapped in `Spawn`.
                        // You can attach modifiers to individual bindings as well.
                        Spawn((Binding::mouse_motion(), Scale::splat(0.01), Negate::all())),
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
                (
                    Action::<CameraInputRotateEnable>::new(),
                    bindings![MouseButton::Right],
                ),
                (
                    Action::<ControlModeSwitch>::new(),
                    bindings![KeyCode::Space],
                )
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
        SpaceshipMarker,
        RigidBody::Dynamic,
        Collider::cylinder(0.5, 1.0),
        ColliderDensity(2.0),
        StableTorquePdController {
            frequency: FREQUENCY,
            damping_ratio: DAMPING_RATIO,
            max_torque: MAX_TORQUE,
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
        RandomSphereOrbit {
            radius: 5.0,
            angular_speed: 5.0,
            center: Vec3::ZERO,
        },
        Transform::from_xyz(0.0, 0.0, -5.0),
        Visibility::Visible,
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.2))),
    ));
}

#[derive(Component, Debug, Clone)]
struct SpaceshipMarker;

#[derive(Component, Debug, Clone)]
struct CameraInputMarker;

#[derive(InputAction)]
#[action_output(Vec2)]
struct CameraInputRotate;

#[derive(InputAction)]
#[action_output(bool)]
struct CameraInputRotateEnable;

#[derive(InputAction)]
#[action_output(f32)]
struct CameraInputZoom;

#[derive(Component, Debug, Clone, Reflect)]
struct CameraInputState {
    rotate_enabled: bool,
}

#[derive(InputAction)]
#[action_output(bool)]
struct ControlModeSwitch;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum ControlMode {
    #[default]
    Torque,
    Smooth,
}

fn update_camera_rotation_input_enabled(
    trigger: Trigger<Fired<CameraInputRotateEnable>>,
    mut q_state: Query<&mut CameraInputState, With<CameraInputMarker>>,
) {
    if let Ok(mut state) = q_state.get_mut(trigger.target()) {
        state.rotate_enabled = trigger.value;
    }
}

fn update_camera_rotation_input(
    trigger: Trigger<Fired<CameraInputRotate>>,
    mut q_input: Query<(&mut OrbitCameraInput, &mut CameraInputState), With<CameraInputMarker>>,
) {
    if let Ok((mut input, mut enabled)) = q_input.get_mut(trigger.target()) {
        if !enabled.rotate_enabled {
            return;
        }

        input.orbit = trigger.value;

        enabled.rotate_enabled = false;
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

fn update_control_mode(
    trigger: Trigger<Started<ControlModeSwitch>>,
    mut control_mode: ResMut<ControlMode>,
) {
    if trigger.value {
        *control_mode = match *control_mode {
            ControlMode::Torque => ControlMode::Smooth,
            ControlMode::Smooth => ControlMode::Torque,
        };
        info!("Switched control mode to {:?}", *control_mode);
    }
}

fn update_spaceship_on_control_mode_changed(
    mut commands: Commands,
    control_mode: Res<ControlMode>,
    q_spaceship: Query<Entity, With<SpaceshipMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !control_mode.is_changed() {
        return;
    }

    for entity in &q_spaceship {
        match *control_mode {
            ControlMode::Torque => {
                commands.entity(entity).despawn();

                // Spawn a spaceship entity (a rectangle with some features to figure out its orientation)
                commands.spawn((
                    Name::new("Spaceship"),
                    SpaceshipMarker,
                    RigidBody::Dynamic,
                    Collider::cylinder(0.5, 1.0),
                    ColliderDensity(2.0),
                    StableTorquePdController {
                        frequency: FREQUENCY,
                        damping_ratio: DAMPING_RATIO,
                        max_torque: MAX_TORQUE,
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
                            Transform::from_xyz(0.0, 0.0, 0.5)
                                .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
                        )
                    ],
                ));
            }
            ControlMode::Smooth => {
                commands.entity(entity).despawn();

                // Spawn a spaceship entity (a rectangle with some features to figure out its orientation)
                commands.spawn((
                    Name::new("Spaceship"),
                    SpaceshipMarker,
                    SmoothTargetRotation {
                        turn_speed: TURN_SPEED,
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
                            Transform::from_xyz(0.0, 0.0, 0.5)
                                .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
                        )
                    ],
                ));
            }
        }
    }
}

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

fn update_spaceship_target_rotation_smooth(
    target: Single<&Transform, With<SpaceshipRotationTargetMarker>>,
    controller: Single<(&mut SmoothTargetRotationTarget, &Transform), With<SmoothTargetRotation>>,
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

fn draw_gizmos_from_spaceship_forward(
    q_spaceship: Query<&Transform, With<SpaceshipMarker>>,
    q_target: Query<&Transform, With<SpaceshipRotationTargetMarker>>,
    mut gizmos: Gizmos,
) {
    let radius = 5.0;

    for transform in &q_spaceship {
        let start = transform.translation;

        let forward = transform.forward();
        let end_forward = start + forward * radius;
        gizmos.line(start, end_forward, Color::srgb(0.2, 0.9, 0.2));

        for target_transform in &q_target {
            let target_pos = target_transform.translation;
            gizmos.line(start, target_pos, Color::srgb(0.9, 0.2, 0.2));

            // Can we draw a line from end to end?
            let to_target = (target_pos - start).normalize_or_zero();
            let end_to_target = start + to_target * radius;
            gizmos.line(end_forward, end_to_target, Color::srgb(0.2, 0.2, 0.9));
        }
    }
}
