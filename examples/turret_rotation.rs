mod helpers;

use avian3d::prelude::*;
use bevy::{core_pipeline::Skybox, prelude::*};
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

    app.add_plugins(SphereRandomOrbitPlugin);
    app.add_plugins(SmoothLookRotationPlugin);
    app.add_plugins(TurretPlugin);
    app.add_systems(
        Update,
        (
            update_turret_target_input,
            // Debugging and visualization systems
            // pdc_turret_color_range_system,
            draw_gizmos_from_turret_forward,
        ),
    );

    app.run();
}

fn update_turret_target_input(
    target: Single<&GlobalTransform, With<PDCTurretTargetMarker>>,
    turret: Single<&mut TurretTargetInput, With<PDCTurretMarker>>,
) {
    let target_transform = target.into_inner();
    let mut turret_target = turret.into_inner();

    **turret_target = target_transform.translation();
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
struct PDCTurretMarker;

#[derive(Component, Clone, Copy, Debug, Reflect)]
struct PDCTurretTargetMarker;

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

    // Spawn a cooler turret entity
    commands.spawn((
        Name::new("Turret"),
        PDCTurretMarker,
        TurretBaseMarker,
        TurretTargetInput(Vec3::ONE),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::Visible,
        DebugAxisMarker,
    )).with_children(|parent| {
        parent.spawn((
            Name::new("Turret Rotator"),
            TurretRotatorMarker,
            SmoothLookRotation {
                yaw_speed: std::f32::consts::PI,   // 180 degrees per second
                pitch_speed: std::f32::consts::PI, // 180 degrees per second
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
            Visibility::Inherited,
            turret_render(&mut meshes, &mut materials),
            DebugAxisMarker,
        ));
    });

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
