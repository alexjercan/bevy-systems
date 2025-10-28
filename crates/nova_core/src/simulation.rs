//! The simulation plugin. This plugin should contain all the gameplay related logic.

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use nova_assets::prelude::*;
use nova_gameplay::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationSystems;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Might want to use observers more for spawning things to avoid ordering issues
        app.add_observer(setup_hud_velocity);
        app.add_observer(remove_hud_velocity);
        app.add_observer(setup_hud_health);
        app.add_observer(remove_hud_health);

        app.add_systems(
            OnEnter(super::GameStates::Simulation),
            (setup_camera_controller, setup_player_input),
        );

        // Setup the input system to get input from the mouse and keyboard.
        app.add_input_context::<PlayerInputMarker>();
        app.add_observer(on_rotation_input);
        app.add_observer(on_rotation_input_completed);
        app.add_observer(on_free_mode_input_started);
        app.add_observer(on_free_mode_input_completed);
        app.add_observer(on_combat_input_started);
        app.add_observer(on_combat_input_completed);

        // On F1 we switch to editor
        // TODO: Use the input system for this
        app.add_systems(
            Update,
            (switch_scene_editor, switch_scene_on_no_player)
                .run_if(in_state(super::GameStates::Simulation)),
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

fn setup_hud_velocity(
    add: On<Add, PlayerSpaceshipMarker>,
    mut commands: Commands,
    q_spaceship: Query<Entity, (With<SpaceshipRootMarker>, With<PlayerSpaceshipMarker>)>,
) {
    let entity = add.entity;
    debug!("setup_hud_velocity: entity {:?}", entity);

    let Ok(spaceship) = q_spaceship.get(entity) else {
        warn!(
            "setup_hud_velocity: entity {:?} not found in q_spaceship",
            entity
        );
        return;
    };

    commands.spawn((
        DespawnOnExit(super::GameStates::Simulation),
        velocity_hud(VelocityHudConfig {
            radius: 5.0,
            target: Some(spaceship),
        }),
    ));
}

fn remove_hud_velocity(
    remove: On<Remove, PlayerSpaceshipMarker>,
    mut commands: Commands,
    q_hud: Query<(Entity, &VelocityHudTargetEntity), With<VelocityHudMarker>>,
) {
    let entity = remove.entity;
    debug!("remove_hud_velocity: entity {:?}", entity);

    for (hud_entity, target) in &q_hud {
        if let Some(target_entity) = **target {
            if target_entity == entity {
                commands.entity(hud_entity).despawn();
            }
        }
    }
}

fn setup_hud_health(
    add: On<Add, PlayerSpaceshipMarker>,
    mut commands: Commands,
    q_spaceship: Query<Entity, (With<SpaceshipRootMarker>, With<PlayerSpaceshipMarker>)>,
) {
    let entity = add.entity;
    debug!("setup_hud_health: entity {:?}", entity);

    let Ok(spaceship) = q_spaceship.get(entity) else {
        warn!(
            "setup_hud_health: entity {:?} not found in q_spaceship",
            entity
        );
        return;
    };

    commands.spawn((
        DespawnOnExit(super::GameStates::Simulation),
        health_hud(HealthHudConfig {
            target: Some(spaceship),
        }),
    ));
}

fn remove_hud_health(
    remove: On<Remove, PlayerSpaceshipMarker>,
    mut commands: Commands,
    q_hud: Query<(Entity, &HealthHudTargetEntity), With<HealthHudMarker>>,
) {
    let entity = remove.entity;
    debug!("remove_hud_health: entity {:?}", entity);

    for (hud_entity, target) in &q_hud {
        if let Some(target_entity) = **target {
            if target_entity == entity {
                commands.entity(hud_entity).despawn();
            }
        }
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

fn setup_player_input(mut commands: Commands) {
    // Spawn a player input controller entity to hold the input from the player
    commands.spawn((
        DespawnOnExit(super::GameStates::Simulation),
        Name::new("Player Input Controller"),
        Transform::default(),
        GlobalTransform::default(),
        PlayerInputMarker,
        actions!(
            PlayerInputMarker[
                (
                    Action::<CameraInputRotate>::new(),
                    Bindings::spawn((
                        // Bevy requires single entities to be wrapped in `Spawn`.
                        // You can attach modifiers to individual bindings as well.
                        Spawn((Binding::mouse_motion(), Scale::splat(0.001), Negate::all())),
                        Axial::right_stick().with((Scale::splat(2.0), Negate::none())),
                    )),
                ),
                (
                    Action::<FreeLookInput>::new(),
                    bindings![KeyCode::AltLeft, GamepadButton::LeftTrigger],
                ),
                (
                    Action::<CombatInput>::new(),
                    bindings![MouseButton::Right],
                ),
            ]
        ),
    ));
}

#[derive(Component, Debug, Clone)]
struct PlayerInputMarker;

#[derive(InputAction)]
#[action_output(Vec2)]
struct CameraInputRotate;

#[derive(InputAction)]
#[action_output(bool)]
struct FreeLookInput;

#[derive(InputAction)]
#[action_output(bool)]
struct CombatInput;

fn on_rotation_input(
    fire: On<Fire<CameraInputRotate>>,
    mut q_input: Query<
        &mut PointRotationInput,
        (
            With<SpaceshipCameraInputMarker>,
            With<SpaceshipRotationInputActiveMarker>,
        ),
    >,
) {
    for mut input in &mut q_input {
        **input = fire.value;
    }
}

fn on_rotation_input_completed(
    _: On<Complete<CameraInputRotate>>,
    mut q_input: Query<&mut PointRotationInput, With<SpaceshipCameraInputMarker>>,
) {
    for mut input in &mut q_input {
        **input = Vec2::ZERO;
    }
}

fn on_free_mode_input_started(
    _: On<Start<FreeLookInput>>,
    mut mode: ResMut<SpaceshipCameraControlMode>,
) {
    *mode = SpaceshipCameraControlMode::FreeLook;
}

fn on_free_mode_input_completed(
    _: On<Complete<FreeLookInput>>,
    mut mode: ResMut<SpaceshipCameraControlMode>,
) {
    *mode = SpaceshipCameraControlMode::Normal;
}

fn on_combat_input_started(
    _: On<Start<CombatInput>>,
    mut mode: ResMut<SpaceshipCameraControlMode>,
) {
    *mode = SpaceshipCameraControlMode::Turret;
}

fn on_combat_input_completed(
    _: On<Complete<CombatInput>>,
    mut mode: ResMut<SpaceshipCameraControlMode>,
) {
    *mode = SpaceshipCameraControlMode::Normal;
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
