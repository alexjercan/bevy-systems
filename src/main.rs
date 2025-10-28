use avian3d::prelude::*;
use bevy::prelude::*;
use clap::Parser;
use nova_protocol::prelude::*;
use rand::prelude::*;

#[derive(Parser)]
#[command(name = "nova_protocol")]
#[command(version = "0.1.0")]
#[command(about = "Simple spaceship editor scene where you can build custom ships", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = AppBuilder::new().build();

    app.add_systems(OnEnter(GameStates::Simulation), setup_simple_scene);

    #[cfg(feature = "debugdump")]
    debugdump(&mut app);

    #[cfg(not(feature = "debugdump"))]
    app.run();
}

// fn setup_grab_cursor(primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
//     let mut primary_cursor_options = primary_cursor_options.into_inner();
//     primary_cursor_options.grab_mode = CursorGrabMode::Locked;
//     primary_cursor_options.visible = false;
// }

pub fn setup_simple_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();

    commands.spawn((
        DespawnOnExit(GameStates::Simulation),
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

        let planet = commands
            .spawn((
                DespawnOnExit(GameStates::Simulation),
                Name::new(format!("Planet {}", i)),
                Transform::from_translation(pos),
                GlobalTransform::default(),
                Mesh3d(meshes.add(Sphere::new(radius))),
                MeshMaterial3d(materials.add(color)),
                Collider::sphere(radius),
                RigidBody::Static,
                Health::new(100.0),
            ))
            .id();

        commands.entity(planet).insert(ExplodeOnDestroy {
            mesh_entity: Some(planet),
            fragment_count: 8,
            ..default()
        });
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

        let satellite = commands
            .spawn((
                DespawnOnExit(GameStates::Simulation),
                Name::new(format!("Satellite {}", i)),
                Transform::from_translation(pos),
                GlobalTransform::default(),
                Mesh3d(meshes.add(Cuboid::new(size, size, size))),
                MeshMaterial3d(materials.add(color)),
                Collider::cuboid(size, size, size),
                ColliderDensity(1.0),
                RigidBody::Dynamic,
                Health::new(100.0),
            ))
            .id();

        commands.entity(satellite).insert(ExplodeOnDestroy {
            mesh_entity: Some(satellite),
            fragment_count: 4,
            ..default()
        });
    }
}
