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
        AsteroidConfig, GameObjectConfig, GameScenarios, MapConfig, ScenarioConfig,
        ScenarioEventConfig, ScenarioId, ScenarioLoad, ScenarioLoaded, ScenarioLoaderPlugin,
        ScenarioScopedMarker, SpaceshipConfig, SpaceshipSectionConfig,
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
pub struct SpaceshipConfig {
    pub id: String,
    pub name: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub health: f32,
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
pub struct ScenarioLoad(pub ScenarioId);

#[derive(Resource, Clone, Debug, Deref, DerefMut, Default, Reflect)]
pub struct ScenarioLoaded(pub Option<ScenarioId>);

#[derive(Component, Debug, Clone, Reflect)]
pub struct ScenarioScopedMarker;

pub struct ScenarioLoaderPlugin;

impl Plugin for ScenarioLoaderPlugin {
    fn build(&self, app: &mut App) {
        debug!("ScenarioLoaderPlugin: build");

        app.init_resource::<ScenarioLoaded>();
        app.add_observer(on_load_scenario);

        app.add_input_context::<ScenarioInputMarker>();
        app.add_observer(on_next_input);
    }
}

fn on_load_scenario(
    load: On<ScenarioLoad>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    scenarios: Res<GameScenarios>,
    mut scenario_loaded: ResMut<ScenarioLoaded>,
    q_scoped: Query<Entity, With<ScenarioScopedMarker>>,
) {
    for entity in q_scoped.iter() {
        commands.entity(entity).despawn();
    }

    let scenario_index = (**load).clone();
    **scenario_loaded = Some(scenario_index.clone());

    // TODO: Here we pretend that we get the scenario from the assets
    let scenario = scenarios.get(&scenario_index).expect("No scenario found");
    info!("Setting up scenario: {}", scenario.name);

    // Fire onstart event
    commands.fire::<OnStartEvent>(OnStartEventInfo::default());

    // Setup scenario input context
    commands.spawn((
        ScenarioScopedMarker,
        Name::new(format!("Scenario Input Context: {}", scenario.name)),
        ScenarioInputMarker,
        actions!(
            ScenarioInputMarker[
                (
                    Name::new("Input: Next Scenario"),
                    Action::<NextScenarioInput>::new(),
                    bindings![KeyCode::Enter, GamepadButton::South]
                )
            ]
        )
    ));

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
                    Mesh3d(meshes.add(Sphere::new(config.radius))),
                    MeshMaterial3d(materials.add(config.color)),
                    Collider::sphere(config.radius),
                    RigidBody::Dynamic,
                    Health::new(config.health),
                    ExplodableEntityMarker,
                ));
            }
            GameObjectConfig::Spaceship(config) => {
                commands
                    .spawn((
                        ScenarioScopedMarker,
                        PlayerSpaceshipMarker,
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
                                        .insert(thruster_section(thruster_config.clone()))
                                        .insert(SpaceshipThrusterInputKey(KeyCode::Space));
                                }
                                SectionKind::Turret(turret_config) => {
                                    section_entity.insert(turret_section(turret_config.clone()));
                                }
                            }
                        }
                    });
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

    // Setup chase camera
    commands.spawn((
        ScenarioScopedMarker,
        Name::new("Chase Camera"),
        Camera3d::default(),
        ChaseCamera::default(),
        SpaceshipCameraControllerMarker,
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
}

#[derive(Component, Debug, Clone)]
struct ScenarioInputMarker;

#[derive(InputAction)]
#[action_output(bool)]
struct NextScenarioInput;

fn on_next_input(
    _: On<Start<NextScenarioInput>,>,
    mut world: ResMut<super::world::NovaEventWorld>,
) {
    let Some(mut next_scenario) = world.next_scenario.clone() else {
        return;
    };

    next_scenario.linger = false;
    world.next_scenario = Some(next_scenario);
}
