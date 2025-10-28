use avian3d::prelude::*;
use bevy::prelude::*;
use clap::Parser;
use nova_protocol::prelude::*;
use rand::prelude::*;

#[derive(Parser)]
#[command(name = "06b_projectile")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how to use the projectile spawner in nova_protocol", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = AppBuilder::new().with_game_plugins(custom_plugin).build();

    app.run();
}

fn custom_plugin(app: &mut App) {
    app.add_plugins((ProjectileSpawnerPlugin::<ExampleProjectileConfig>::default(),));

    app.add_systems(
        OnEnter(GameStates::Simulation),
        (setup_spawner, setup_camera, setup_simple_scene),
    );

    app.add_observer(on_despawn_projectile);
    app.add_systems(Update, on_projectile_input);
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct ExampleProjectileSpawnerMarker;

fn on_projectile_input(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    q_spawner: Query<Entity, With<ExampleProjectileSpawnerMarker>>,
) {
    for spawner in &q_spawner {
        if mouse.pressed(MouseButton::Left) {
            commands.trigger(SpawnProjectile::<ExampleProjectileConfig> {
                entity: spawner,
                initial_velocity: Vec3::ZERO,
                ..default()
            });
        }
    }
}

fn setup_spawner(mut commands: Commands) {
    commands.spawn((
        Name::new("Projectile Spawner"),
        ExampleProjectileSpawnerMarker,
        projectile_spawner(ProjectileSpawnerConfig {
            fire_rate: 100.0,
            projectile: ExampleProjectileConfig { ..default() },
            ..default()
        }),
        Transform::from_xyz(0.0, 0.0, 0.0),
        TransformChainWorld::default(),
        Visibility::Inherited,
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

#[derive(Clone, Debug, Reflect)]
pub struct ExampleProjectileConfig {
    pub lifetime: f32,
    pub final_message: String,
}

impl Default for ExampleProjectileConfig {
    fn default() -> Self {
        Self {
            lifetime: 5.0,
            final_message: "Projectile expired".to_string(),
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct ExampleProjectileMarker;

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct ProjectileFinalMessage(pub String);

impl ProjectileBundle for ExampleProjectileConfig {
    fn projectile_bundle(&self) -> impl Bundle {
        println!(
            "Creating projectile bundle with lifetime {} and final message '{}'",
            self.lifetime, self.final_message
        );

        (
            ExampleProjectileMarker,
            TempEntity(self.lifetime),
            ProjectileFinalMessage(self.final_message.clone()),
        )
    }
}

fn on_despawn_projectile(
    despawn: On<Despawn, ExampleProjectileMarker>,
    q_projectiles: Query<&ProjectileFinalMessage, With<ExampleProjectileMarker>>,
) {
    let entity = despawn.entity;
    if let Ok(final_message) = q_projectiles.get(entity) {
        println!("Projectile despawned: {}", **final_message);
    }
}
