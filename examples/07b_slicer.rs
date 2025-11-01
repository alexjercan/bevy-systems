use avian3d::prelude::*;
use bevy::prelude::*;
use clap::Parser;
use nova_protocol::prelude::*;
use rand::prelude::*;

#[derive(Parser)]
#[command(name = "07b_slicer")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how to play with the mesh slicer in nova_protocol", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = AppBuilder::new().with_game_plugins(custom_plugin).build();

    app.run();
}

fn custom_plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameStates::Simulation),
        (setup_health_entity, setup_camera, setup_simple_scene, setup_event_handlers),
    );

    app.add_observer(on_click_damage_health);
    app.add_observer(on_fragment_added);
}

fn on_click_damage_health(
    click: On<Pointer<Press>>,
    mut commands: Commands,
    q_health: Query<&Health>,
) {
    if click.button != PointerButton::Primary {
        return;
    }

    let entity = click.entity;

    if q_health.get(entity).is_ok() {
        commands.trigger(HealthApplyDamage {
            target: click.entity,
            source: None,
            amount: 10.0,
        });
    }
}

fn on_fragment_added(
    add: On<Add, FragmentMeshMarker>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh_entity = commands.entity(add.entity).insert((
        MeshMaterial3d(materials.add(Color::srgb(rand::random(), rand::random(), rand::random()))),
        Health::new(10.0),
    )).id();

    commands.entity(mesh_entity).insert(ExplodableMesh(vec![mesh_entity]));
}

fn setup_health_entity(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh_entity = commands
        .spawn((
            Name::new("Health Entity"),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::Visible,
            Mesh3d(meshes.add(Sphere::new(3.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.1, 0.1))),
            Health::new(10.0),
            Collider::sphere(3.0),
        ))
        .id();

    commands.entity(mesh_entity).insert(ExplodableMesh(vec![mesh_entity]));
}

fn setup_event_handlers(mut commands: Commands) {
    commands.spawn((
        Name::new("OnDestroyedEvent Handler"),
        EventHandler::<NovaEventWorld>::new::<OnDestroyedEvent>()
            .with_filter(EntityFilter::default())
            .with_action(InsertComponentAction(CollisionLayers::NONE))
            .with_action(InsertComponentAction(ExplodeMesh {
                fragment_count: 2,
                force_multiplier_range: (1.0, 2.0)
            }))
            .with_action(EntityDespawnAction),
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
