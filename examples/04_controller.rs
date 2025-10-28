use avian3d::prelude::*;
use bevy::prelude::*;
use clap::Parser;
use nova_protocol::prelude::*;
use rand::prelude::*;

#[derive(Parser)]
#[command(name = "04_controller")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how to spawn a basic controller in nova_protocol", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = AppBuilder::new().with_game_plugins(custom_plugin).build();

    app.run();
}

fn custom_plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameStates::Simulation),
        (setup_spaceship, setup_camera, setup_simple_scene),
    );

    app.add_systems(
        Update,
        update_spaceship_target_rotation_torque.before(SpaceshipSystems),
    );
}

#[derive(Component, Debug, Clone)]
struct SpaceshipRotationInputMarker;

fn update_spaceship_target_rotation_torque(
    point_rotation: Single<&GlobalTransform, With<SpaceshipRotationInputMarker>>,
    controller: Single<&mut ControllerSectionRotationInput, With<ControllerSectionMarker>>,
) {
    let rotation = point_rotation.rotation();
    let mut controller_target = controller.into_inner();
    **controller_target = rotation;
}

fn setup_spaceship(mut commands: Commands) {
    let entity = commands
        .spawn((spaceship_root(SpaceshipConfig { ..default() }),))
        .id();

    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            base_section(BaseSectionConfig {
                name: "Basic Controller Section".to_string(),
                description: "A basic controller section for spaceships.".to_string(),
                mass: 1.0,
            }),
            controller_section(ControllerSectionConfig {
                frequency: 4.0,
                damping_ratio: 4.0,
                max_torque: 100.0,
                ..default()
            }),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
    });

    commands.spawn((
        Name::new("Spaceship Rotation Target"),
        SpaceshipRotationInputMarker,
        Transform::from_xyz(0.0, 0.0, 0.0),
        #[cfg(feature = "debug")]
        DebugAxisMarker,
    ));
}

fn setup_camera(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        Name::new("Main Camera"),
        Camera3d::default(),
        WASDCameraController,
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        SkyboxConfig {
            cubemap: game_assets.cubemap.clone(),
            brightness: 1000.0,
        },
    ));
}

fn setup_simple_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();

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
