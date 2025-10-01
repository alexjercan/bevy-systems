mod helpers;

use avian3d::{math::*, prelude::*};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_systems::prelude::*;
use clap::Parser;
use helpers::prelude::*;

#[derive(Parser)]
#[command(name = "turret_rotation")]
#[command(version = "0.1")]
#[command(about = "Example for the Turret Rotation", long_about = None)]
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

    app.add_plugins(SphereRandomOrbitPlugin);
    app.add_plugins(SmoothLookRotationPlugin);
    app.add_systems(
        Update,
        (
            update_pdc_turret_target_system,
            // Debugging and visualization systems
            // pdc_turret_color_range_system,
            draw_gizmos_from_turret_forward,
        ),
    );

    app.run();
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
struct PDCTurretMarker;

#[derive(Component, Clone, Copy, Debug, Reflect)]
struct PDCTurretTargetMarker;

fn update_pdc_turret_target_system(
    target: Single<&Transform, With<PDCTurretTargetMarker>>,
    q_turret: Single<(&Transform, &mut SmoothLookRotationTarget), With<PDCTurretMarker>>,
) {
    let target_transform = target.into_inner();
    let (turret_transform, mut look_target) = q_turret.into_inner();

    let direction = (target_transform.translation - turret_transform.translation).normalize_or_zero();
    if direction.length_squared() < 1e-6 {
        return;
    }

    let (yaw, pitch, _) = Quat::from_rotation_arc(Vec3::NEG_Z, direction).to_euler(EulerRot::YXZ);
    *look_target = SmoothLookRotationTarget {
        yaw,
        pitch,
    };
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    ));

    // Spawn a cooler turret entity
    commands.spawn((
        Name::new("Turret"),
        PDCTurretMarker,
        SmoothLookRotation {
            yaw_speed: PI * 2.0,
            pitch_speed: PI * 2.0,
            min_pitch: Some(-std::f32::consts::FRAC_PI_6),
            max_pitch: Some(std::f32::consts::FRAC_PI_3),
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::Visible,
        children![
            // Base
            (
                Name::new("Turret Base"),
                Transform::default(),
                Mesh3d(meshes.add(Cylinder::new(0.6, 0.3))),
                MeshMaterial3d(materials.add(Color::srgb(0.3, 0.3, 0.3))),
            ),
            // Yaw rotor / mount
            (
                Name::new("Turret Rotor"),
                Transform::from_xyz(0.0, 0.15, 0.0),
                Mesh3d(meshes.add(Cylinder::new(0.4, 0.1))),
                MeshMaterial3d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
            ),
            // Sphere for pivot point
            (
                Name::new("Turret Pivot"),
                Transform::from_xyz(0.0, -0.2, 0.0),
                Mesh3d(meshes.add(Sphere::new(0.5))),
                MeshMaterial3d(materials.add(Color::srgb(0.7, 0.7, 0.7))),
            ),
            // Main Barrel
            (
                Name::new("Turret Barrel"),
                Transform::from_xyz(0.0, 0.0, -0.8),
                Mesh3d(meshes.add(Cuboid::new(0.15, 0.15, 1.2))),
                MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.7))),
                children![
                    // Barrel tip
                    (
                        Name::new("Barrel Tip"),
                        Transform::from_xyz(0.0, 0.0, -0.6),
                        Mesh3d(meshes.add(Cone::new(0.1, 0.2))),
                        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.2, 0.2))),
                    ),
                    // Optional second barrel (for twin cannons)
                    (
                        Name::new("Second Barrel"),
                        Transform::from_xyz(0.2, 0.0, -0.4),
                        Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.8))),
                        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.7))),
                    ),
                ],
            ),
            // Small detail lights on the base
            (
                Name::new("Base Lights"),
                Transform::from_xyz(0.35, 0.0, 0.0),
                Mesh3d(meshes.add(Sphere::new(0.05))),
                MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.2))),
            ),
            (
                Name::new("Base Lights 2"),
                Transform::from_xyz(-0.35, 0.0, 0.0),
                Mesh3d(meshes.add(Sphere::new(0.05))),
                MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.2))),
            ),
        ],
        DebugAxisMarker,
    ));

    // Spawn a target entity to visualize the target rotation
    commands.spawn((
        Name::new("Turret Target"),
        PDCTurretTargetMarker,
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

fn draw_gizmos_from_turret_forward(
    q_turret: Query<&Transform, With<PDCTurretMarker>>,
    q_target: Query<&Transform, With<PDCTurretTargetMarker>>,
    mut gizmos: Gizmos,
) {
    let radius = 5.0;

    for transform in &q_turret {
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

// fn pdc_turret_color_range_system(
//     mut commands: Commands,
//     mut q_turret: Query<(&Transform, &Children), With<PDCTurretMarker>>,
//     q_mesh: Query<&MeshMaterial3d<StandardMaterial>, With<PDCTurretBaseMarker>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     for (transform, children) in &mut q_turret {
//         // direction to target
//         let dir = (**target - transform.translation).normalize_or_zero();
//         if dir.length_squared() < 1e-6 {
//             continue;
//         }
//
//         // compute yaw/pitch in world space (same as your turret system)
//         let (_, pitch, _) = Quat::from_rotation_arc(-Vec3::Z, dir).to_euler(EulerRot::YXZ);
//
//         // check if pitch is in range
//         let in_range = pitch >= turret.min_pitch && pitch <= turret.max_pitch;
//
//         // set color based on range
//         let color = if in_range {
//             Color::srgb(0.2, 0.7, 0.2) // green if in range
//         } else {
//             Color::srgb(0.7, 0.2, 0.2) // red if out of range
//         };
//
//         // apply to all child meshes of the turret
//         for child in children.iter() {
//             if let Ok(_) = q_mesh.get(child) {
//                 commands
//                     .entity(child)
//                     .insert(MeshMaterial3d(materials.add(color)));
//             }
//         }
//     }
// }
