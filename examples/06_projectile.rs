use avian3d::prelude::*;
use bevy::prelude::*;
use clap::Parser;
use nova_protocol::prelude::*;
use rand::prelude::*;

#[derive(Parser)]
#[command(name = "06_projectile")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how to spawn a basic projectile in nova_protocol", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = new_gui_app();

    app.add_plugins(ProjectilePlugin::<BulletProjectileConfig>::default());
    app.add_plugins(BulletProjectilePlugin { render: true });
    app.add_plugins(TempEntityPlugin);
    app.add_plugins(ProjectileVelocityPlugin);

    app.add_systems(
        OnEnter(GameStates::Playing),
        (setup_projectile_spawner, setup_camera, setup_simple_scene),
    );

    app.add_systems(Update, on_projectile_input);

    app.run();
}

fn on_projectile_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    q_spawner: Query<Entity, With<ProjectileSpawnerMarker<BulletProjectileConfig>>>,
) {
    for spawner_entity in &q_spawner {
        if keys.pressed(KeyCode::KeyQ) {
            commands.trigger(SpawnProjectile {
                entity: spawner_entity,
            });
        }
    }
}

fn setup_projectile_spawner(mut commands: Commands) {
    commands.spawn((
        projectile_spawner(ProjectileSpawnerConfig::<BulletProjectileConfig> {
            muzzle_speed: 200.0,
            muzzle_offset: Vec3::new(0.0, 0.0, -2.0),
            fire_rate: 50.0,
            transform: Transform::default(),
            projectile: BulletProjectileConfig {
                lifetime: 5.0,
                render_mesh: None,
            },
        }),
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
