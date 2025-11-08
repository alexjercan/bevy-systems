use avian3d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};
use bevy_common_systems::prelude::*;
use bevy_enhanced_input::prelude::*;

use super::{
    actions::EventActionConfig,
    events::{OnStartEvent, OnStartEventInfo},
    filters::EventFilterConfig,
    world::NovaEventWorld,
};
use crate::prelude::*;

pub mod prelude {
    pub use super::{
        AIControllerConfig, AsteroidConfig, CurrentScenario, GameObjectConfig, GameScenarios,
        LoadScenario, LoadScenarioById, MapConfig, PlayerControllerConfig, ScenarioConfig,
        ScenarioEventConfig, ScenarioId, ScenarioLoaded, ScenarioLoaderPlugin,
        ScenarioScopedMarker, SpaceshipConfig, SpaceshipController, SpaceshipSectionConfig,
        UnloadScenario,
    };
}

pub type ScenarioId = String;

#[derive(Resource, Clone, Debug, Deref, DerefMut, Default)]
pub struct GameScenarios(pub HashMap<ScenarioId, ScenarioConfig>);

#[derive(Clone, Debug)]
pub struct ScenarioConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub map: MapConfig,
    pub events: Vec<ScenarioEventConfig>,
}

#[derive(Clone, Debug)]
pub struct MapConfig {
    pub cubemap: Handle<Image>,
    pub objects: Vec<GameObjectConfig>,
}

#[derive(Clone, Debug)]
pub enum GameObjectConfig {
    Asteroid(AsteroidConfig),
    Spaceship(SpaceshipConfig),
}

#[derive(Clone, Debug)]
pub struct AsteroidConfig {
    pub id: String,
    pub name: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub radius: f32,
    pub color: Color,
    pub health: f32,
}

#[derive(Clone, Debug)]
pub enum SpaceshipController {
    None,
    Player(PlayerControllerConfig),
    AI(AIControllerConfig),
}

#[derive(Clone, Debug)]
pub struct PlayerControllerConfig {
    // TODO: Add some kind of input mapping from Section ID to input actions
    // TODO: Add Section ID in the SpaceshipSectionConfig as String maybe
    // pub input_mapping: HashMap<>,
}

#[derive(Clone, Debug)]
pub struct AIControllerConfig {}

#[derive(Clone, Debug)]
pub struct SpaceshipConfig {
    pub id: String,
    pub name: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub health: f32,
    pub controller: SpaceshipController,
    pub sections: Vec<SpaceshipSectionConfig>,
}

#[derive(Clone, Debug)]
pub struct SpaceshipSectionConfig {
    pub position: Vec3,
    pub rotation: Quat,
    // NOTE: Maybe in the future this will be a Handle and in the .cfg file it will be represented
    // by an ID.
    pub config: SectionConfig,
}

#[derive(Clone, Debug)]
pub struct ScenarioEventConfig {
    pub name: EventConfig,
    pub filters: Vec<EventFilterConfig>,
    pub actions: Vec<EventActionConfig>,
}

#[derive(Event, Clone, Debug, Deref, DerefMut, Default, Reflect)]
pub struct LoadScenarioById(pub ScenarioId);

#[derive(Event, Clone, Debug, Deref, DerefMut)]
pub struct LoadScenario(pub ScenarioConfig);

#[derive(Event, Clone, Debug)]
pub struct UnloadScenario;

#[derive(Event, Clone, Debug, Deref, DerefMut)]
pub struct ScenarioLoaded(pub ScenarioConfig);

#[derive(Resource, Clone, Debug, Deref, DerefMut, Default)]
pub struct CurrentScenario(pub Option<ScenarioConfig>);

#[derive(Component, Debug, Clone, Reflect)]
pub struct ScenarioScopedMarker;

pub struct ScenarioLoaderPlugin;

impl Plugin for ScenarioLoaderPlugin {
    fn build(&self, app: &mut App) {
        debug!("ScenarioLoaderPlugin: build");

        app.init_resource::<CurrentScenario>();
        app.add_observer(on_load_scenario_id);
        app.add_observer(on_load_scenario);
        app.add_observer(on_player_spaceship_spawned);
        app.add_observer(on_player_spaceship_destroyed);

        app.add_observer(on_add_entity);

        app.add_input_context::<ScenarioInputMarker>();
        app.add_observer(on_next_input);
        app.add_observer(unload_scenario);
    }
}

fn unload_scenario(
    _: On<UnloadScenario>,
    mut commands: Commands,
    q_scoped: Query<Entity, With<ScenarioScopedMarker>>,
    mut current_scenario: ResMut<CurrentScenario>,
    mut world: ResMut<NovaEventWorld>,
) {
    for entity in q_scoped.iter() {
        commands.entity(entity).despawn();
    }

    **current_scenario = None;
    world.clear();
}

fn on_load_scenario_id(
    load: On<LoadScenarioById>,
    mut commands: Commands,
    scenarios: Res<GameScenarios>,
) {
    let scenario_index = (**load).clone();
    let scenario = scenarios.get(&scenario_index).expect("No scenario found");
    commands.trigger(LoadScenario(scenario.clone()));
}

fn on_load_scenario(
    load: On<LoadScenario>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut current_scenario: ResMut<CurrentScenario>,
    q_scoped: Query<Entity, With<ScenarioScopedMarker>>,
) {
    // NOTE: Clean up any existing scenario-scoped entities
    // TODO: Maybe in the future we want to filter more specifically in case we keep other
    // scenario-scoped entities around (e.g the player spaceship or similar)
    for entity in q_scoped.iter() {
        commands.entity(entity).despawn();
    }

    let scenario = (**load).clone();
    **current_scenario = Some(scenario.clone());
    info!("Setting up scenario: {}", scenario.name);

    // Setup Scenario Camera
    commands.spawn((
        ScenarioScopedMarker,
        Name::new("Scenario Camera"),
        ScenarioCameraMarker,
        Camera3d::default(),
        WASDCameraController,
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        SkyboxConfig {
            cubemap: scenario.map.cubemap.clone(),
            brightness: 1000.0,
        },
    ));

    // Setup directional light
    commands.spawn((
        ScenarioScopedMarker,
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

    // Setup scenario input context
    commands.spawn((
        ScenarioScopedMarker,
        Name::new(format!("Scenario Input Context: {}", scenario.name)),
        ScenarioInputMarker,
        actions!(
            ScenarioInputMarker[(
                Name::new("Input: Next Scenario"),
                Action::<NextScenarioInput>::new(),
                bindings![KeyCode::Enter, GamepadButton::South]
            )]
        ),
    ));

    // Fire onstart event
    commands.fire::<OnStartEvent>(OnStartEventInfo);

    // Spawn all objects in the scenario
    for object in scenario.map.objects.iter() {
        match object {
            GameObjectConfig::Asteroid(config) => {
                commands.spawn((
                    ScenarioScopedMarker,
                    Name::new(config.name.clone()),
                    EntityId::new(config.id.clone()),
                    EntityTypeName::new("asteroid"),
                    Transform::from_translation(config.position).with_rotation(config.rotation),
                    Collider::sphere(config.radius),
                    RigidBody::Dynamic,
                    Health::new(config.health),
                    ExplodableEntityMarker,
                    children![(
                        Name::new("Asteroid Mesh"),
                        Transform::from_scale(Vec3::splat(config.radius)),
                        Mesh3d(meshes.add(octahedron_sphere(3))),
                        MeshMaterial3d(materials.add(config.color)),
                    )],
                ));
            }
            GameObjectConfig::Spaceship(config) => {
                let entity = commands
                    .spawn((
                        ScenarioScopedMarker,
                        SpaceshipRootMarker,
                        Name::new(config.name.clone()),
                        EntityId::new(config.id.clone()),
                        EntityTypeName::new("spaceship"),
                        Transform::from_translation(config.position).with_rotation(config.rotation),
                        RigidBody::Dynamic,
                        Visibility::Visible,
                        Health::new(config.health),
                        ExplodableEntityMarker,
                    ))
                    .with_children(|parent| {
                        for section in config.sections.iter() {
                            let mut section_entity = parent.spawn((
                                base_section(section.config.base.clone()),
                                Transform::from_translation(section.position)
                                    .with_rotation(section.rotation),
                            ));

                            match &section.config.kind {
                                SectionKind::Hull(hull_config) => {
                                    section_entity.insert(hull_section(hull_config.clone()));
                                }
                                SectionKind::Controller(controller_config) => {
                                    section_entity
                                        .insert(controller_section(controller_config.clone()));
                                }
                                SectionKind::Thruster(thruster_config) => {
                                    section_entity
                                        .insert(thruster_section(thruster_config.clone()));

                                    match config.controller {
                                        SpaceshipController::None => {}
                                        SpaceshipController::Player(_) => {
                                            // TODO: Something like
                                            // let key = config.input_mapping.get(&section_id);
                                            section_entity
                                                .insert(SpaceshipThrusterInputKey(KeyCode::Space));
                                        }
                                        SpaceshipController::AI(_) => {}
                                    }
                                }
                                SectionKind::Turret(turret_config) => {
                                    section_entity.insert(turret_section(turret_config.clone()));

                                    match config.controller {
                                        SpaceshipController::None => {}
                                        SpaceshipController::Player(_) => {
                                            section_entity
                                                .insert(SpaceshipTurretInputKey(MouseButton::Left));
                                        }
                                        SpaceshipController::AI(_) => {}
                                    }
                                }
                            }
                        }
                    })
                    .id();

                match config.controller {
                    SpaceshipController::None => {}
                    SpaceshipController::Player(_) => {
                        commands.entity(entity).insert(PlayerSpaceshipMarker);
                    }
                    SpaceshipController::AI(_) => {
                        commands.entity(entity).insert(AISpaceshipMarker);
                    }
                }
            }
        }
    }

    // Setup scenario events
    for event in scenario.events.iter() {
        let mut event_handler = EventHandler::<NovaEventWorld>::from(event.name);
        for filter in event.filters.iter() {
            event_handler.add_filter(filter.clone());
        }
        for action in event.actions.iter() {
            event_handler.add_action(action.clone());
        }
        commands.spawn((
            ScenarioScopedMarker,
            Name::new(format!("Event Handler: {:?}", event.name)),
            event_handler,
        ));
    }

    commands.trigger(ScenarioLoaded(scenario));
}

fn on_add_entity(
    add: On<Add, Name>,
    mut commands: Commands,
    current_scenario: Res<CurrentScenario>,
) {
    if let Some(scenario) = &**current_scenario {
        trace!(
            "on_add_entity: Added entity {:?} in scenario {:?}",
            add.entity,
            scenario.name
        );

        commands.entity(add.entity).insert(ScenarioScopedMarker);
    }
}

#[derive(Component, Debug, Clone)]
struct ScenarioInputMarker;

#[derive(InputAction)]
#[action_output(bool)]
struct NextScenarioInput;

fn on_next_input(_: On<Start<NextScenarioInput>>, mut world: ResMut<super::world::NovaEventWorld>) {
    let Some(mut next_scenario) = world.next_scenario.clone() else {
        return;
    };

    next_scenario.linger = false;
    world.next_scenario = Some(next_scenario);
}

#[derive(Component, Debug, Clone)]
struct ScenarioCameraMarker;

fn on_player_spaceship_spawned(
    add: On<Add, PlayerSpaceshipMarker>,
    mut commands: Commands,
    current_scenario: Res<CurrentScenario>,
    camera: Single<(Entity, &Transform), With<ScenarioCameraMarker>>,
) {
    trace!("on_player_spaceship_spawned: {:?}", add.entity);

    let Some(scenario) = &**current_scenario else {
        warn!("on_player_spaceship_spawned: no scenario loaded");
        return;
    };
    let (camera, transform) = camera.into_inner();

    // Replace the existing scenario camera with a chase camera
    commands.entity(camera).despawn();
    commands.spawn((
        ScenarioScopedMarker,
        Name::new("Scenario Camera"),
        ScenarioCameraMarker,
        Camera3d::default(),
        ChaseCamera::default(),
        SpaceshipCameraControllerMarker,
        *transform,
        SkyboxConfig {
            cubemap: scenario.map.cubemap.clone(),
            brightness: 1000.0,
        },
    ));
}

fn on_player_spaceship_destroyed(
    add: On<Add, DestroyedMarker>,
    mut commands: Commands,
    current_scenario: Res<CurrentScenario>,
    camera: Single<(Entity, &Transform), With<SpaceshipCameraControllerMarker>>,
    spaceship: Single<Entity, With<PlayerSpaceshipMarker>>,
) {
    trace!("on_player_spaceship_destroyed: {:?}", add.entity);
    if add.entity != spaceship.into_inner() {
        return;
    }

    let Some(scenario) = &**current_scenario else {
        warn!("on_player_spaceship_despawned: no scenario loaded");
        return;
    };
    let (camera, transform) = camera.into_inner();

    // Replace the chase camera with the scenario camera
    commands.entity(camera).despawn();
    commands.spawn((
        ScenarioScopedMarker,
        Name::new("Scenario Camera"),
        ScenarioCameraMarker,
        Camera3d::default(),
        WASDCameraController,
        *transform,
        SkyboxConfig {
            cubemap: scenario.map.cubemap.clone(),
            brightness: 1000.0,
        },
    ));
}
