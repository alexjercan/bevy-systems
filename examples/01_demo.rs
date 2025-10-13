mod helpers;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use clap::Parser;
use helpers::*;
use nova_protocol::prelude::*;

#[derive(Parser)]
#[command(name = "spaceship_demo_01")]
#[command(version = "0.1")]
#[command(about = "Demo for the first version", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = new_gui_app();
    app.add_plugins(EnhancedInputPlugin);

    // Helper plugins
    app.add_plugins(GameAssetsPlugin);
    app.add_plugins(GameSkyboxPlugin);
    app.add_plugins(WASDCameraControllerPlugin);

    // We need to enable the physics plugins to have access to RigidBody and other components.
    // We will also disable gravity for this example, since we are in space, duh.
    app.add_plugins(PhysicsPlugins::default().set(PhysicsInterpolationPlugin::interpolate_all()));
    if cfg!(feature = "debug") {
        app.add_plugins(PhysicsDebugPlugin::default());
    }
    app.insert_resource(Gravity::ZERO);

    // Add sections plugins
    app.add_plugins(SpaceshipPlugin);

    app.add_systems(
        OnEnter(GameStates::Playing),
        (
            setup_scene,
            setup_simple_scene,
            setup_spaceship,
        ),
    );

    app.add_input_context::<PlayerInputMarker>();
    app.add_observer(setup_input);
    app.add_observer(on_thruster_input);
    app.add_observer(on_thruster_input_completed);

    app.run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Name::new("WASD Camera"),
        WASDCameraController,
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn setup_spaceship(mut commands: Commands) {
    commands.spawn((
        spaceship_root(SpaceshipConfig { ..default() }),
        children![
            (hull_section(HullSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            }),),
            (hull_section(HullSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..default()
            }),),
            (hull_section(HullSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                ..default()
            }),),
            (engine_section(EngineSectionConfig {
                thrust_magnitude: 1.0,
                transform: Transform::from_xyz(0.0, 0.0, 2.0),
                ..default()
            }), ThrusterInputKey(KeyCode::Digit1)),
            (engine_section(EngineSectionConfig {
                thrust_magnitude: 0.1,
                transform: Transform::from_xyz(-1.0, 0.0, 1.0)
                    .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
                ..default()
            }), ThrusterInputKey(KeyCode::Digit2)),
            (engine_section(EngineSectionConfig {
                thrust_magnitude: 0.1,
                transform: Transform::from_xyz(1.0, 0.0, 1.0)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                ..default()
            }), ThrusterInputKey(KeyCode::Digit3)),
            (engine_section(EngineSectionConfig {
                thrust_magnitude: 0.1,
                transform: Transform::from_xyz(-1.0, 0.0, -1.0)
                    .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
                ..default()
            }), ThrusterInputKey(KeyCode::Digit2)),
            (engine_section(EngineSectionConfig {
                thrust_magnitude: 0.1,
                transform: Transform::from_xyz(1.0, 0.0, -1.0)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                ..default()
            }), ThrusterInputKey(KeyCode::Digit3)),
        ],
    ));
}

#[derive(Component, Debug, Clone, Deref, DerefMut, Reflect)]
struct ThrusterInputKey(KeyCode);

fn setup_input(insert: On<Insert, ThrusterInputKey>, mut commands: Commands, q_key: Query<&ThrusterInputKey>) {
    let entity = insert.entity;
    let ThrusterInputKey(key) = q_key.get(entity).unwrap();
    commands.entity(entity).insert((
        PlayerInputMarker,
        actions!(
            PlayerInputMarker[
                (
                    Action::<ThrusterInput>::new(),
                    bindings![*key],
                ),
            ]
        ),
    ));
}

#[derive(Component, Debug, Clone)]
struct PlayerInputMarker;

#[derive(InputAction)]
#[action_output(bool)]
struct ThrusterInput;

fn on_thruster_input(fire: On<Fire<ThrusterInput>>, mut q_input: Query<&mut EngineThrustInput>) {
    let entity = fire.event().event_target();
    if let Ok(mut input) = q_input.get_mut(entity) {
        **input = 1.0;
    }
}

fn on_thruster_input_completed(
    complete: On<Complete<ThrusterInput>>,
    mut q_input: Query<&mut EngineThrustInput>,
) {
    let entity = complete.event().event_target();
    if let Ok(mut input) = q_input.get_mut(entity) {
        **input = 0.0;
    }
}
