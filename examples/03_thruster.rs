use avian3d::prelude::*;
use bevy::prelude::*;
use clap::Parser;
use nova_protocol::prelude::*;
use rand::prelude::*;

#[derive(Parser)]
#[command(name = "03_thruster")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how to spawn a basic thruster in nova_protocol", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = new_gui_app();

    app.add_systems(
        OnEnter(GameStates::Playing),
        (setup_spaceship, setup_camera, setup_simple_scene),
    );

    app.add_systems(Update, on_thruster_input);

    app.run();
}

#[derive(Component, Debug, Clone, Deref, DerefMut, Reflect)]
struct ThrusterInputKey(KeyCode);

fn on_thruster_input(
    mut q_input: Query<(&mut ThrusterSectionInput, &ThrusterInputKey)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut input, key) in &mut q_input {
        if keys.pressed(key.0) {
            **input = 1.0;
        } else {
            **input = 0.0;
        }
    }
}

fn setup_spaceship(mut commands: Commands, game_assets: Res<GameAssets>) {
    let entity = commands
        .spawn((spaceship_root(SpaceshipConfig { ..default() }),))
        .id();

    let cube_size = 1;
    for x in -cube_size..=cube_size {
        for y in -cube_size..=cube_size {
            for z in -cube_size..=cube_size {
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((hull_section(HullSectionConfig {
                        transform: Transform::from_xyz(
                            x as f32 * 1.0,
                            y as f32 * 1.0,
                            z as f32 * 1.0,
                        ),
                        render_mesh: Some(game_assets.hull_01.clone()),
                        ..default()
                    }),));
                });
            }
        }
    }

    let z = cube_size + 1;
    for x in -cube_size..=cube_size {
        for y in -cube_size..=cube_size {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    thruster_section(ThrusterSectionConfig {
                        magnitude: 1.0,
                        transform: Transform::from_xyz(
                            x as f32 * 1.0,
                            y as f32 * 1.0,
                            z as f32 * 1.0,
                        ),
                        ..default()
                    }),
                    ThrusterInputKey(KeyCode::Digit1),
                ));
            });
        }
    }
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
