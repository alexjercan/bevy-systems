mod helpers;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use clap::Parser;
use helpers::*;
use nova_protocol::prelude::*;

#[derive(Parser)]
#[command(name = "01_demo_thruster")]
#[command(version = "0.1")]
#[command(about = "Demo for the first version for thrusters", long_about = None)]
struct Cli;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum SceneStates {
    #[default]
    None,
    Simple,
    Thruster,
    Complex,
    Spinner,
    Exagerating,
}

fn main() {
    let _ = Cli::parse();
    let mut app = new_gui_app();
    app.add_plugins(EnhancedInputPlugin);

    app.init_state::<SceneStates>();

    // Helper plugins
    app.add_plugins(GameAssetsPlugin);
    if cfg!(feature = "debug") {
        app.add_plugins(DebugGizmosPlugin);
    }
    app.add_plugins(WASDCameraControllerPlugin);

    // We need to enable the physics plugins to have access to RigidBody and other components.
    // We will also disable gravity for this example, since we are in space, duh.
    app.add_plugins(PhysicsPlugins::default().set(PhysicsInterpolationPlugin::interpolate_all()));
    if cfg!(feature = "debug") {
        app.add_plugins(PhysicsDebugPlugin::default());
    }
    app.insert_resource(Gravity::ZERO);

    // Render Plugins
    app.add_plugins(SkyboxPlugin);
    app.add_plugins(PostProcessingDefaultPlugin);

    // Add sections plugins
    app.add_plugins(SpaceshipPlugin { render: true });

    app.add_systems(
        OnEnter(GameStates::Playing),
        (setup_scene, setup_simple_scene),
    );

    app.add_systems(
        OnEnter(SceneStates::Simple),
        setup_spaceship_simple.run_if(in_state(GameStates::Playing)),
    );
    app.add_systems(
        OnEnter(SceneStates::Thruster),
        setup_spaceship_thruster.run_if(in_state(GameStates::Playing)),
    );
    app.add_systems(
        OnEnter(SceneStates::Complex),
        setup_spaceship_complex.run_if(in_state(GameStates::Playing)),
    );
    app.add_systems(
        OnEnter(SceneStates::Spinner),
        setup_spaceship_spinner.run_if(in_state(GameStates::Playing)),
    );
    app.add_systems(
        OnEnter(SceneStates::Exagerating),
        setup_spaceship_exagerating.run_if(in_state(GameStates::Playing)),
    );
    app.add_systems(
        OnEnter(GameStates::Playing),
        |mut state: ResMut<NextState<SceneStates>>| {
            state.set(SceneStates::Simple);
        },
    );
    app.add_systems(Update, switch_scene.run_if(in_state(GameStates::Playing)));

    app.add_systems(Update, on_thruster_input);

    app.run();
}

fn setup_scene(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        Name::new("WASD Camera"),
        WASDCameraController,
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        SkyboxConfig {
            cubemap: game_assets.cubemap.clone(),
            brightness: 1000.0,
        },
    ));
}

fn setup_spaceship_simple(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        DespawnOnExit(SceneStates::Simple),
        spaceship_root(SpaceshipConfig { ..default() }),
        children![(hull_section(HullSectionConfig {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            render_mesh: Some(game_assets.hull_01.clone()),
            ..default()
        }),),],
    ));
}

fn setup_spaceship_thruster(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        DespawnOnExit(SceneStates::Thruster),
        spaceship_root(SpaceshipConfig { ..default() }),
        children![
            (hull_section(HullSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                render_mesh: Some(game_assets.hull_01.clone()),
                ..default()
            }),),
            (
                thruster_section(ThrusterSectionConfig {
                    magnitude: 1.0,
                    transform: Transform::from_xyz(0.0, 0.0, 1.0),
                    ..default()
                }),
                ThrusterInputKey(KeyCode::Digit1)
            ),
        ],
    ));
}

fn setup_spaceship_complex(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        DespawnOnExit(SceneStates::Complex),
        spaceship_root(SpaceshipConfig { ..default() }),
        children![
            (hull_section(HullSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                render_mesh: Some(game_assets.hull_01.clone()),
                ..default()
            }),),
            (hull_section(HullSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                render_mesh: Some(game_assets.hull_01.clone()),
                ..default()
            }),),
            (hull_section(HullSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                render_mesh: Some(game_assets.hull_01.clone()),
                ..default()
            }),),
            (
                thruster_section(ThrusterSectionConfig {
                    magnitude: 1.0,
                    transform: Transform::from_xyz(0.0, 0.0, 2.0),
                    ..default()
                }),
                ThrusterInputKey(KeyCode::Digit1)
            ),
            (
                thruster_section(ThrusterSectionConfig {
                    magnitude: 0.1,
                    transform: Transform::from_xyz(-1.0, 0.0, 1.0)
                        .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
                    ..default()
                }),
                ThrusterInputKey(KeyCode::Digit2)
            ),
            (
                thruster_section(ThrusterSectionConfig {
                    magnitude: 0.1,
                    transform: Transform::from_xyz(1.0, 0.0, 1.0)
                        .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                    ..default()
                }),
                ThrusterInputKey(KeyCode::Digit3)
            ),
            (
                thruster_section(ThrusterSectionConfig {
                    magnitude: 0.1,
                    transform: Transform::from_xyz(-1.0, 0.0, -1.0)
                        .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
                    ..default()
                }),
                ThrusterInputKey(KeyCode::Digit3)
            ),
            (
                thruster_section(ThrusterSectionConfig {
                    magnitude: 0.1,
                    transform: Transform::from_xyz(1.0, 0.0, -1.0)
                        .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                    ..default()
                }),
                ThrusterInputKey(KeyCode::Digit2)
            ),
        ],
    ));
}

fn setup_spaceship_spinner(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        DespawnOnExit(SceneStates::Spinner),
        spaceship_root(SpaceshipConfig { ..default() }),
        children![
            (hull_section(HullSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                render_mesh: Some(game_assets.hull_01.clone()),
                ..default()
            }),),
            (hull_section(HullSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                render_mesh: Some(game_assets.hull_01.clone()),
                ..default()
            }),),
            (hull_section(HullSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                render_mesh: Some(game_assets.hull_01.clone()),
                ..default()
            }),),
            (
                thruster_section(ThrusterSectionConfig {
                    magnitude: 1.0,
                    transform: Transform::from_xyz(1.0, 0.0, 1.0)
                        .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                    ..default()
                }),
                ThrusterInputKey(KeyCode::Digit1)
            ),
            (
                thruster_section(ThrusterSectionConfig {
                    magnitude: 1.0,
                    transform: Transform::from_xyz(-1.0, 0.0, -1.0)
                        .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
                    ..default()
                }),
                ThrusterInputKey(KeyCode::Digit1)
            ),
        ],
    ));
}

fn setup_spaceship_exagerating(mut commands: Commands, game_assets: Res<GameAssets>) {
    let entity = commands
        .spawn((
            DespawnOnExit(SceneStates::Exagerating),
            spaceship_root(SpaceshipConfig { ..default() }),
        ))
        .id();

    let cube_size = 5;
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

fn switch_scene(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<NextState<SceneStates>>) {
    if keys.just_pressed(KeyCode::F1) {
        state.set(SceneStates::Simple);
    } else if keys.just_pressed(KeyCode::F2) {
        state.set(SceneStates::Thruster);
    } else if keys.just_pressed(KeyCode::F3) {
        state.set(SceneStates::Complex);
    } else if keys.just_pressed(KeyCode::F4) {
        state.set(SceneStates::Spinner);
    } else if keys.just_pressed(KeyCode::F5) {
        state.set(SceneStates::Exagerating);
    }
}
