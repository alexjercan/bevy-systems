use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use nova_gameplay::{self, bevy_common_systems, prelude::*};

#[derive(Default, Clone, Debug)]
pub(crate) struct CorePlugin {
    pub render: bool,
}

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        // We need to enable the physics plugins to have access to RigidBody and other components.
        // We will also disable gravity for this example, since we are in space, duh.
        app.add_plugins(PhysicsPlugins::default().with_collision_hooks::<TurretProjectileHooks>());
        app.add_plugins(PhysicsPickingPlugin);
        app.insert_resource(Gravity::ZERO);

        // FIXME: For now we disable particle effects on wasm because it's not working
        #[cfg(not(target_family = "wasm"))]
        app.add_plugins(bevy_hanabi::HanabiPlugin);

        // Bevy Common Systems - WASD Camera
        app.add_plugins(bevy_common_systems::prelude::WASDCameraPlugin);
        app.add_plugins(bevy_common_systems::prelude::WASDCameraControllerPlugin);
        // Chase Camera Plugin to have a 3rd person camera following the spaceship
        app.add_plugins(bevy_common_systems::prelude::ChaseCameraPlugin);
        // Bevy Common Systems - Rendering
        app.add_plugins(bevy_common_systems::prelude::SkyboxPlugin);
        app.add_plugins(bevy_common_systems::prelude::PostProcessingDefaultPlugin);
        // Point Rotation Plugin to convert linear movement to a target rotation
        app.add_plugins(bevy_common_systems::prelude::PointRotationPlugin);
        // for debug to have a random orbiting object
        app.add_plugins(bevy_common_systems::prelude::SphereRandomOrbitPlugin);
        // Rotation Plugin for the turret facing direction
        app.add_plugins(bevy_common_systems::prelude::SmoothLookRotationPlugin);
        // Sphere Orbit Plugin
        app.add_plugins(bevy_common_systems::prelude::SphereOrbitPlugin);
        app.add_plugins(bevy_common_systems::prelude::DirectionalSphereOrbitPlugin);
        // Other helper plugins
        app.add_plugins(bevy_common_systems::prelude::TempEntityPlugin);
        app.add_plugins(bevy_common_systems::prelude::DespawnEntityPlugin);
        app.add_plugins(bevy_common_systems::prelude::ExplodeMeshPlugin);
        // Core Mechanics
        app.add_plugins(bevy_common_systems::prelude::CollisionDamagePlugin);
        app.add_plugins(bevy_common_systems::prelude::HealthPlugin);

        // UI Plugins
        app.add_plugins(nova_gameplay::bevy_common_systems::prelude::StatusBarPlugin);

        // Core Plugins for simulation
        app.add_plugins(nova_gameplay::spaceship::SpaceshipPlugin {
            render: self.render,
        });
        app.add_plugins(nova_gameplay::damage::DamagePlugin);
        app.add_plugins(nova_gameplay::modding::NovaEventsPlugin);

        // Diagnostics
        if !app.is_plugin_added::<bevy::diagnostic::FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
        }

        // Configure system Sets
        app.configure_sets(
            Update,
            SpaceshipSystems::Input.run_if(in_state(super::GameStates::Simulation)),
        );

        // Fire OnStartEvent when entering Simulation state
        app.add_systems(
            OnEnter(super::GameStates::Simulation),
            (|mut commands: Commands| {
                commands.fire::<OnStartEvent>(OnStartEventInfo::default());
            },),
        );

        app.add_observer(setup_hud_velocity);
        app.add_observer(remove_hud_velocity);
        app.add_observer(setup_hud_health);
        app.add_observer(remove_hud_health);

        // Setup the input system to get input from the mouse and keyboard.
        app.add_input_context::<PlayerInputMarker>();

        app.add_observer(setup_player_input);
        app.add_observer(on_rotation_input);
        app.add_observer(on_rotation_input_completed);
        app.add_observer(on_free_mode_input_started);
        app.add_observer(on_free_mode_input_completed);
        app.add_observer(on_combat_input_started);
        app.add_observer(on_combat_input_completed);

        // Cleanup entities when exiting Simulation state
        app.add_observer(|add: On<Add, EntityTypeName>, mut commands: Commands| {
            let entity = add.entity;
            commands
                .entity(entity)
                .insert(DespawnOnExit(super::GameStates::Simulation));
        });
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

fn setup_player_input(add: On<Add, ChaseCamera>, mut commands: Commands) {
    // TODO: Make this an observer that adds the input system only when we spawn the player in the
    // world game.

    let _entity = add.entity;

    // Spawn a player input controller entity to hold the input from the player
    commands.spawn((
        DespawnOnExit(super::GameStates::Simulation),
        Name::new("Player Input Controller"),
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
