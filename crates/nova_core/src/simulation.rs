//! The simulation plugin. This plugin should contain all the gameplay related logic.

use avian3d::prelude::*;
use bevy::prelude::*;
use nova_assets::prelude::*;
use nova_gameplay::prelude::*;
use rand::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationSystems;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Might want to use observers more for spawning things to avoid ordering issues

        app.add_systems(
            OnEnter(super::GameStates::Simulation),
            (
                setup_simple_scene,
                setup_camera_controller,
                setup_spaceship_sections,
                switch_scene_on_no_player.run_if(run_once),
            ),
        );

        // On F1 we switch to editor
        // TODO: Use the input system for this
        app.add_systems(
            Update,
            switch_scene_editor.run_if(in_state(super::GameStates::Simulation)),
        );

        app.add_systems(
            OnExit(super::GameStates::Simulation),
            |mut q_thruster: Query<&mut ThrusterSectionInput, With<SpaceshipThrusterInputKey>>| {
                for mut input in &mut q_thruster {
                    **input = 0.0;
                }
            },
        );
        app.add_systems(
            OnExit(super::GameStates::Simulation),
            |mut commands: Commands, q_fragment: Query<Entity, With<FragmentMeshMarker>>| {
                for fragment in &q_fragment {
                    commands.entity(fragment).despawn();
                }
            },
        );
    }
}

fn setup_simple_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();

    commands.spawn((
        DespawnOnExit(super::GameStates::Simulation),
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
            DespawnOnExit(super::GameStates::Simulation),
            Name::new(format!("Asteroid {}", i)),
            EntityTypeName::new("asteroid"),
            Transform::from_translation(pos),
            GlobalTransform::default(),
            Mesh3d(meshes.add(Sphere::new(radius))),
            MeshMaterial3d(materials.add(color)),
            Collider::sphere(radius),
            RigidBody::Dynamic,
            Health::new(100.0),
            ExplodableEntityMarker,
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
            DespawnOnExit(super::GameStates::Simulation),
            Name::new(format!("Junk {}", i)),
            EntityTypeName::new("junk"),
            Transform::from_translation(pos),
            GlobalTransform::default(),
            Mesh3d(meshes.add(Cuboid::new(size, size, size))),
            MeshMaterial3d(materials.add(color)),
            Collider::cuboid(size, size, size),
            ColliderDensity(1.0),
            RigidBody::Dynamic,
            Health::new(100.0),
            ExplodableEntityMarker,
        ));
    }
}

fn setup_spaceship_sections(
    mut commands: Commands,
    q_spaceship: Query<Entity, With<SpaceshipRootMarker>>,
) {
    for spaceship in &q_spaceship {
        commands
            .entity(spaceship)
            .insert((EntityTypeName::new("spaceship"), ExplodableEntityMarker));
    }
}

fn setup_camera_controller(mut commands: Commands, game_assets: Res<GameAssets>) {
    // Spawn a 3D camera with a chase camera component
    commands.spawn((
        DespawnOnExit(super::GameStates::Simulation),
        Name::new("Chase Camera"),
        Camera3d::default(),
        ChaseCamera::default(),
        SpaceshipCameraControllerMarker,
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        SkyboxConfig {
            cubemap: game_assets.cubemap.clone(),
            brightness: 1000.0,
        },
    ));
}

fn switch_scene_editor(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<super::GameStates>>,
) {
    if keys.just_pressed(KeyCode::F1) {
        debug!("switch_scene_editor: F1 pressed, switching to Editor state.");
        state.set(super::GameStates::Editor);
    }
}

fn switch_scene_on_no_player(
    mut state: ResMut<NextState<super::GameStates>>,
    q_spaceship: Query<&Health, (With<SpaceshipRootMarker>, With<PlayerSpaceshipMarker>)>,
) {
    if q_spaceship.is_empty() {
        debug!("switch_scene_on_no_player: No player spaceship found, switching to Editor state.");
        state.set(super::GameStates::Editor);
    }
}
