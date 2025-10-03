#![allow(dead_code)]

use bevy::{
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};
use bevy_asset_loader::prelude::*;
use avian3d::prelude::*;
use rand::prelude::*;


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

        app.add_systems(OnEnter(GameStates::Playing), setup);
    }
}

#[derive(AssetCollection, Resource, Clone)]
pub struct GameAssets {
    #[asset(path = "textures/cubemap.png")]
    pub cubemap: Handle<Image>,
}

fn setup(game_assets: Res<GameAssets>, mut images: ResMut<Assets<Image>>) {
    let image = images.get_mut(&game_assets.cubemap).unwrap();
    if image.texture_descriptor.array_layer_count() == 1 {
        image.reinterpret_stacked_2d_as_array(image.height() / image.width());
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
    }
}

/// A plugin that draws debug gizmos for entities.
pub struct DebugGizmosPlugin;

impl Plugin for DebugGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_debug_gizmos_axis);
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

/// A simple scene setup with random planets and satellites.
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
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -std::f32::consts::FRAC_PI_2, 0.0, 0.0)),
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
