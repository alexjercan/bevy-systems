use avian3d::prelude::*;
use bevy::prelude::*;
use clap::Parser;
use nova_protocol::prelude::*;
use rand::prelude::*;

#[derive(Parser)]
#[command(name = "05_turret")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how to spawn a basic turret in nova_protocol", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = AppBuilder::new().with_game_plugins(custom_plugin).build();

    app.run();
}

fn custom_plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameStates::Simulation),
        (
            setup_target,
            setup_spaceship,
            setup_camera,
            setup_simple_scene,
        ),
    );

    app.add_systems(
        Update,
        (sync_random_orbit_state, update_turret_target_input).chain(),
    );
}

fn sync_random_orbit_state(
    mut q_orbit: Query<
        (&mut Transform, &RandomSphereOrbitOutput),
        Changed<RandomSphereOrbitOutput>,
    >,
) {
    for (mut transform, output) in &mut q_orbit {
        transform.translation = **output;
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
struct PDCTurretTargetMarker;

fn update_turret_target_input(
    target: Single<&GlobalTransform, With<PDCTurretTargetMarker>>,
    mut q_turret: Query<&mut TurretSectionTargetInput, With<TurretSectionMarker>>,
) {
    let target_transform = target.into_inner();

    for mut turret in &mut q_turret {
        **turret = Some(target_transform.translation());
    }
}

fn setup_target(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Turret Target"),
        PDCTurretTargetMarker,
        RandomSphereOrbit {
            radius: 5.0,
            angular_speed: 5.0,
            center: Vec3::ZERO,
            ..default()
        },
        Transform::default(),
        Visibility::Visible,
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.2))),
    ));
}

fn setup_spaceship(mut commands: Commands, game_assets: Res<GameAssets>) {
    let entity = commands
        .spawn((spaceship_root(SpaceshipConfig { ..default() }),))
        .id();

    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            base_section(BaseSectionConfig {
                name: "Basic Turret Section".to_string(),
                description: "A basic turret section for spaceships.".to_string(),
                mass: 1.0,
            }),
            turret_section(TurretSectionConfig {
                render_mesh_yaw: Some(game_assets.turret_yaw_01.clone()),
                render_mesh_pitch: Some(game_assets.turret_pitch_01.clone()),
                pitch_offset: Vec3::new(0.0, 0.332706, 0.303954),
                render_mesh_barrel: Some(game_assets.turret_barrel_01.clone()),
                barrel_offset: Vec3::new(0.0, 0.128437, -0.110729),
                ..default()
            }),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
        parent.spawn((
            base_section(BaseSectionConfig {
                name: "Basic Turret Section".to_string(),
                description: "A basic turret section for spaceships.".to_string(),
                mass: 1.0,
            }),
            turret_section(TurretSectionConfig { ..default() }),
            Transform::from_xyz(1.0, 0.0, 0.0),
        ));
    });
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
