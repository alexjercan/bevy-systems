use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

use super::{
    actions::EventActionConfig,
    events::{OnStartEvent, OnStartEventInfo},
    filters::EventFilterConfig,
    world::NovaEventWorld,
};
use crate::{
    prelude::{EntityId, EntityTypeName, EventConfig, SectionConfig},
    spaceship::prelude::*,
};

pub mod prelude {
    pub use super::{
        AsteroidConfig, GameObjectConfig, GameScenarios, MapConfig, ScenarioConfig,
        ScenarioEventConfig, ScenarioLoad, ScenarioLoaderPlugin, ScenarioStates, SpaceshipConfig,
        SpaceshipSectionConfig,
    };
}

#[derive(Resource, Clone, Debug, Deref, DerefMut, Default)]
pub struct GameScenarios(pub Vec<ScenarioConfig>);

#[derive(Clone, Debug)]
pub struct ScenarioConfig {
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
pub struct ScenarioLoad(pub usize);

/// Scenario States
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub struct ScenarioStates(pub Option<usize>);

pub struct ScenarioLoaderPlugin;

impl Plugin for ScenarioLoaderPlugin {
    fn build(&self, app: &mut App) {
        debug!("ScenarioLoaderPlugin: build");

        app.init_state::<ScenarioStates>();
        app.add_observer(on_load_scenario);
    }
}

fn on_load_scenario(
    load: On<ScenarioLoad>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    scenarios: Res<GameScenarios>,
    mut scenario_state: ResMut<NextState<ScenarioStates>>,
) {
    let scenario_index = **load;
    scenario_state.set(ScenarioStates(Some(scenario_index)));

    // TODO: Here we pretend that we get the scenario from the assets
    let scenario = scenarios.get(scenario_index).expect("No scenario found");
    info!("Setting up scenario: {}", scenario.name);

    // Fire onstart event
    commands.fire::<OnStartEvent>(OnStartEventInfo::default());

    // Spawn all objects in the scenario
    for object in scenario.map.objects.iter() {
        match object {
            GameObjectConfig::Asteroid(config) => {
                commands.spawn((
                    DespawnOnExit(ScenarioStates(Some(scenario_index))),
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
                        DespawnOnExit(ScenarioStates(Some(scenario_index))),
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
        commands.spawn(event_handler);
    }

    // Setup chase camera
    commands.spawn((
        DespawnOnExit(ScenarioStates(Some(scenario_index))),
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
        DespawnOnExit(ScenarioStates(Some(scenario_index))),
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
