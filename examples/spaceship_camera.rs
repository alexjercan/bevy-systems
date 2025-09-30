//! In this example, I want to demo how to use StableTorquePdController to rotate a spaceship to
//! follow the mouse cursor. The spaceship will rotate to face the mouse cursor when moved.

use avian3d::{math::*, prelude::*};
use bevy::{
    core_pipeline::Skybox,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};
use bevy_asset_loader::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_systems::prelude::*;
use clap::Parser;

#[derive(Parser)]
#[command(name = "spaceship_camera")]
#[command(version = "0.1")]
#[command(about = "Example for the 3rd person camera controller", long_about = None)]
struct Cli;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameStates {
    #[default]
    Loading,
    Playing,
}

fn main() {
    let _ = Cli::parse();

    let mut app = new_gui_app();

    app.init_state::<GameStates>();
    app.add_loading_state(
        LoadingState::new(GameStates::Loading)
            .continue_to_state(GameStates::Playing)
            .load_collection::<GameAssets>(),
    );

    app.add_plugins(PhysicsPlugins::default());
    app.insert_resource(Gravity::ZERO);
    app.add_systems(OnEnter(GameStates::Playing), (setup, setup_scene));

    app.add_plugins(EnhancedInputPlugin);
    app.add_input_context::<PlayerInputMarker>();
    app.add_observer(update_camera_rotation_input);
    app.add_observer(update_thruster_input);

    app.add_plugins(ChaseCameraPlugin);
    app.add_plugins(PointRotationPlugin);
    app.add_plugins(StableTorquePdControllerPlugin);
    app.add_systems(
        Update,
        (
            update_spaceship_target_rotation_torque,
            update_chase_camera_input,
        ),
    );

    app.add_systems(Update, gizmos_draw_crosshair);

    app.run();
}

const FREQUENCY: f32 = 2.0;
const DAMPING_RATIO: f32 = 2.0;
const MAX_TORQUE: f32 = 1.0;

#[derive(AssetCollection, Resource, Clone)]
pub struct GameAssets {
    #[asset(path = "textures/cubemap.png")]
    pub cubemap: Handle<Image>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>,
    mut images: ResMut<Assets<Image>>,
) {
    let image = images.get_mut(&game_assets.cubemap).unwrap();
    if image.texture_descriptor.array_layer_count() == 1 {
        image.reinterpret_stacked_2d_as_array(image.height() / image.width());
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
    }

    // Spawn a 3D camera with a chase camera component
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
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

    // Spawn a player input controller entity to hold the input and target for the chase camera
    commands.spawn((
        Name::new("Player Input Controller"),
        Transform::default(),
        GlobalTransform::default(),
        PointRotation::default(),
        PlayerInputMarker,
        actions!(
            PlayerInputMarker[
                (
                    Action::<CameraInputRotate>::new(),
                    Bindings::spawn((
                        // Bevy requires single entities to be wrapped in `Spawn`.
                        // You can attach modifiers to individual bindings as well.
                        Spawn((Binding::mouse_motion(), Scale::splat(0.001), Negate::none())),
                        Axial::right_stick().with((Scale::splat(2.0), Negate::none())),
                    )),
                ),
                (
                    Action::<ThrusterInput>::new(),
                    bindings![KeyCode::KeyW, GamepadButton::RightTrigger],
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
        SpaceshipMarker,
        ChaseCameraTargetMarker,
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
    ));
}

#[derive(Component, Debug, Clone)]
struct SpaceshipMarker;

#[derive(Component, Debug, Clone)]
struct PlayerInputMarker;

#[derive(InputAction)]
#[action_output(Vec2)]
struct CameraInputRotate;

#[derive(InputAction)]
#[action_output(bool)]
struct ThrusterInput;

fn update_camera_rotation_input(
    trigger: Trigger<Fired<CameraInputRotate>>,
    mut q_input: Query<&mut PointRotationInput, With<PlayerInputMarker>>,
) {
    if let Ok(mut input) = q_input.get_mut(trigger.target()) {
        **input = trigger.value;
    }
}

fn update_thruster_input(
    _: Trigger<Fired<ThrusterInput>>,
    mut q_spaceship: Query<(&mut ExternalImpulse, &GlobalTransform), With<SpaceshipMarker>>,
) {
    for (mut force, spaceship_transform) in &mut q_spaceship {
        let thrust_direction = spaceship_transform.forward();
        let thrust_magnitude = 1.0;

        force.apply_impulse(thrust_direction * thrust_magnitude);
    }
}

fn update_spaceship_target_rotation_torque(
    target: Single<&Transform, With<PlayerInputMarker>>,
    controller: Single<&mut StableTorquePdControllerTarget, With<StableTorquePdController>>,
) {
    let target_transform = target.into_inner();
    let mut controller_target = controller.into_inner();

    **controller_target = target_transform.rotation;
}

fn update_chase_camera_input(
    controller: Single<&GlobalTransform, With<PlayerInputMarker>>,
    mut q_camera_input: Query<&mut ChaseCameraAnchor>,
) {
    let transform = controller.into_inner();
    let anchor_rot = transform.rotation();

    for mut input in q_camera_input.iter_mut() {
        input.achor_rot = anchor_rot;
    }
}

fn gizmos_draw_crosshair(
    mut gizmos: Gizmos,
    spaceship: Single<&GlobalTransform, With<SpaceshipMarker>>,
    controller: Single<&GlobalTransform, With<PlayerInputMarker>>,
) {
    // Draw the forward of the spaceship
    let spaceship_transform = spaceship.into_inner();
    let spaceship_forward = spaceship_transform.forward();
    gizmos.line(
        spaceship_transform.translation(),
        spaceship_transform.translation() + spaceship_forward * 5.0,
        Color::srgb(0.9, 0.9, 0.1),
    );

    // Draw the forward of the controller (where the spaceship is trying to face)
    let controller_transform = controller.into_inner();
    let controller_forward = controller_transform.forward();
    gizmos.line(
        spaceship_transform.translation(),
        spaceship_transform.translation() + controller_forward * 5.0,
        Color::srgb(0.1, 0.9, 0.1),
    );
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    use rand::prelude::*;
    let mut rng = rand::rng();

    // Planets (bigger spheres, sparse)
    for i in 0..20 {
        let pos = Vec3::new(
            rng.random_range(-100.0..100.0),
            rng.random_range(-20.0..20.0),
            rng.random_range(-100.0..100.0),
        );
        let radius = rng.random_range(2.0..6.0);
        let color = Color::srgb(
            rng.random_range(0.0..1.0),
            rng.random_range(0.0..1.0),
            rng.random_range(0.0..1.0),
        );

        commands.spawn((
            Name::new(format!("Planet {}", i)),
            Transform::from_translation(pos),
            GlobalTransform::default(),
            Mesh3d(meshes.add(Sphere::new(radius))),
            MeshMaterial3d(materials.add(color)),
        ));
    }

    // Satellites (small cubes, further away from planets)
    for i in 0..40 {
        let pos = Vec3::new(
            rng.random_range(-120.0..120.0),
            rng.random_range(-30.0..30.0),
            rng.random_range(-120.0..120.0),
        );
        let size = rng.random_range(0.5..1.0);
        let color = Color::srgb(
            rng.random_range(0.6..1.0),
            rng.random_range(0.6..1.0),
            rng.random_range(0.0..0.6),
        );

        commands.spawn((
            Name::new(format!("Satellite {}", i)),
            Transform::from_translation(pos),
            GlobalTransform::default(),
            Mesh3d(meshes.add(Cuboid::new(size, size, size))),
            MeshMaterial3d(materials.add(color)),
        ));
    }
}
