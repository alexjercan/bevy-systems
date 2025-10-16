#![allow(dead_code)]

use avian3d::prelude::*;
use bevy::{
    // core_pipeline::{tonemapping::Tonemapping, Skybox},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    // post_process::bloom::Bloom,
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_enhanced_input::prelude::*;
use rand::prelude::*;

use crate::prelude::{WASDCamera, WASDCameraInput, WASDCameraPlugin};

/// Game states for the application.
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameStates {
    #[default]
    Loading,
    Playing,
}

/// A plugin that loads game assets and sets up the game.
pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        // Setup the asset loader to load assets during the loading state.
        app.init_state::<GameStates>();
        app.add_loading_state(
            LoadingState::new(GameStates::Loading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource, Clone)]
pub struct GameAssets {
    #[asset(path = "textures/cubemap.png")]
    pub cubemap: Handle<Image>,
    #[asset(path = "gltf/hull-01.glb#Scene0")]
    pub hull_01: Handle<Scene>,
}

/// A plugin that draws debug gizmos for entities.
pub struct DebugGizmosPlugin;

impl Plugin for DebugGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin::default());
        app.insert_resource(WireframeConfig {
            global: true,
            ..default()
        });

        app.add_systems(Update, (draw_debug_gizmos_axis, toggle_wireframe));
    }
}

#[derive(Component)]
pub struct DebugAxisMarker;

fn draw_debug_gizmos_axis(
    mut gizmos: Gizmos,
    q_transform: Query<&GlobalTransform, With<DebugAxisMarker>>,
) {
    // Draw the xyz axis of all entities with a GlobalTransform
    for transform in &q_transform {
        let origin = transform.translation();
        let x_axis = transform.rotation() * Vec3::X * 2.0;
        let y_axis = transform.rotation() * Vec3::Y * 2.0;
        let z_axis = transform.rotation() * Vec3::NEG_Z * 2.0;

        gizmos.line(origin, origin + x_axis, Color::srgb(0.9, 0.1, 0.1));
        gizmos.line(origin, origin + y_axis, Color::srgb(0.1, 0.9, 0.1));
        gizmos.line(origin, origin + z_axis, Color::srgb(0.1, 0.1, 0.9));
    }
}

fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::F11) {
        wireframe_config.global = !wireframe_config.global;
    }
}

/// A simple scene setup_skybox_asset with random planets and satellites.
pub fn setup_simple_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();

    // Spawn a light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_2,
            0.0,
            0.0,
        )),
        GlobalTransform::default(),
    ));

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
            Collider::sphere(radius),
            RigidBody::Static,
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
            Collider::cuboid(size, size, size),
            ColliderDensity(1.0),
            RigidBody::Dynamic,
        ));
    }
}

/// A plugin that sets up WASD and mouse controls for a camera.
pub struct WASDCameraControllerPlugin;

impl Plugin for WASDCameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WASDCameraPlugin);

        app.add_input_context::<WASDCameraInputMarker>();

        app.add_observer(setup_wasd_camera);
        app.add_observer(on_wasd_input);
        app.add_observer(on_wasd_input_completed);
        app.add_observer(on_mouse_input);
        app.add_observer(on_mouse_input_completed);
        app.add_observer(on_enable_look_input);
        app.add_observer(on_enable_look_input_completed);
        app.add_observer(on_vertical_input);
        app.add_observer(on_vertical_input_completed);
    }
}

#[derive(Component, Debug, Clone)]
struct WASDCameraInputMarker;

#[derive(InputAction)]
#[action_output(Vec2)]
struct WASDCameraInputMove;

#[derive(InputAction)]
#[action_output(Vec2)]
struct WASDCameraInputLook;

#[derive(InputAction)]
#[action_output(bool)]
struct WASDCameraInputEnableLook;

#[derive(InputAction)]
#[action_output(f32)]
struct WASDCameraInputVertical;

#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut, Reflect)]
struct WASDCameraLookEnabled(bool);

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
pub struct WASDCameraController;

fn setup_wasd_camera(insert: On<Insert, WASDCameraController>, mut commands: Commands) {
    commands.entity(insert.entity).insert((
        Camera3d::default(),
        WASDCamera {
            wasd_sensitivity: 0.1,
            ..default()
        },
        WASDCameraLookEnabled(false),
        WASDCameraInputMarker,
        actions!(
            WASDCameraInputMarker[
                (
                    Action::<WASDCameraInputMove>::new(),
                    Bindings::spawn((
                        Cardinal::wasd_keys().with(Scale::splat(1.0)),
                        Axial::left_stick().with(Scale::splat(1.0)),
                    )),
                ),
                (
                    Action::<WASDCameraInputLook>::new(),
                    Bindings::spawn((
                        // Bevy requires single entities to be wrapped in `Spawn`.
                        // You can attach modifiers to individual bindings as well.
                        Spawn((Binding::mouse_motion(), Scale::splat(0.01), Negate::none())),
                        Axial::right_stick().with((Scale::splat(1.0), Negate::none())),
                    )),
                ),
                (
                    Action::<WASDCameraInputEnableLook>::new(),
                    bindings![MouseButton::Right],
                ),
                (
                    Action::<WASDCameraInputVertical>::new(),
                    Bindings::spawn((
                        Bidirectional::<Binding, Binding> {
                            positive: KeyCode::Space.into(),
                            negative: KeyCode::ShiftLeft.into(),
                        },
                    )),
                ),
            ]
        ),
    ));
}

fn on_wasd_input(fire: On<Fire<WASDCameraInputMove>>, mut q_input: Query<&mut WASDCameraInput>) {
    for mut input in &mut q_input {
        input.wasd = fire.value;
    }
}

fn on_wasd_input_completed(
    _: On<Complete<WASDCameraInputMove>>,
    mut q_input: Query<&mut WASDCameraInput>,
) {
    for mut input in &mut q_input {
        input.wasd = Vec2::ZERO;
    }
}

fn on_mouse_input(
    fire: On<Fire<WASDCameraInputLook>>,
    mut q_input: Query<(&mut WASDCameraInput, &WASDCameraLookEnabled)>,
) {
    for (mut input, enabled) in &mut q_input {
        if !**enabled {
            continue;
        }

        input.pan = fire.value;
    }
}

fn on_mouse_input_completed(
    _: On<Complete<WASDCameraInputLook>>,
    mut q_input: Query<&mut WASDCameraInput>,
) {
    for mut input in &mut q_input {
        input.pan = Vec2::ZERO;
    }
}

fn on_enable_look_input(
    _: On<Fire<WASDCameraInputEnableLook>>,
    mut q_look_enabled: Query<&mut WASDCameraLookEnabled>,
) {
    for mut look_enabled in &mut q_look_enabled {
        **look_enabled = true;
    }
}

fn on_enable_look_input_completed(
    _: On<Complete<WASDCameraInputEnableLook>>,
    mut q_look_enabled: Query<(&mut WASDCameraInput, &mut WASDCameraLookEnabled)>,
) {
    for (mut input, mut look_enabled) in &mut q_look_enabled {
        input.pan = Vec2::ZERO;
        **look_enabled = false;
    }
}

fn on_vertical_input(
    fire: On<Fire<WASDCameraInputVertical>>,
    mut q_input: Query<&mut WASDCameraInput>,
) {
    for mut input in &mut q_input {
        input.vertical = fire.value;
    }
}

fn on_vertical_input_completed(
    _: On<Complete<WASDCameraInputVertical>>,
    mut q_input: Query<&mut WASDCameraInput>,
) {
    for mut input in &mut q_input {
        input.vertical = 0.0;
    }
}
