use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

use crate::prelude::*;

pub mod prelude {
    pub use super::{
        AIControllerConfig, AsteroidConfig, DebugMessageActionConfig, EventActionConfig,
        ScenarioObjectConfig, NextScenarioActionConfig, ObjectiveActionConfig,
        ObjectiveCompleteActionConfig, PlayerControllerConfig, SpaceshipConfig,
        SpaceshipController, SpaceshipSectionConfig, VariableSetActionConfig,
    };
}

#[derive(Clone, Debug)]
pub enum EventActionConfig {
    DebugMessage(DebugMessageActionConfig),
    VariableSet(VariableSetActionConfig),
    Objective(ObjectiveActionConfig),
    ObjectiveComplete(ObjectiveCompleteActionConfig),
    SpawnScenarioObject(ScenarioObjectConfig),
    NextScenario(NextScenarioActionConfig),
}

impl EventAction<NovaEventWorld> for EventActionConfig {
    fn action(&self, world: &mut NovaEventWorld, info: &GameEventInfo) {
        match self {
            EventActionConfig::DebugMessage(config) => {
                config.action(world, info);
            }
            EventActionConfig::VariableSet(config) => {
                config.action(world, info);
            }
            EventActionConfig::Objective(config) => {
                config.action(world, info);
            }
            EventActionConfig::ObjectiveComplete(config) => {
                config.action(world, info);
            }
            EventActionConfig::SpawnScenarioObject(config) => {
                config.action(world, info);
            }
            EventActionConfig::NextScenario(config) => {
                config.action(world, info);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct VariableSetActionConfig {
    pub key: String,
    pub expression: VariableExpressionNode,
}

impl EventAction<NovaEventWorld> for VariableSetActionConfig {
    fn action(&self, world: &mut NovaEventWorld, _: &GameEventInfo) {
        match self.expression.evaluate(world) {
            Ok(literal) => {
                world.variables.insert(self.key.clone(), literal);
            }
            Err(e) => {
                // TODO: Proper error handling
                error!(
                    "VariableSetActionConfig: failed to evaluate expression for key '{}': {:?}",
                    self.key, e
                );
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DebugMessageActionConfig {
    pub message: String,
}

impl EventAction<NovaEventWorld> for DebugMessageActionConfig {
    fn action(&self, _: &mut NovaEventWorld, _: &GameEventInfo) {
        debug!("Event Action Message: {}", self.message);
    }
}

#[derive(Clone, Debug, Default)]
pub struct NextScenarioActionConfig {
    pub scenario_id: String,
    pub linger: bool,
}

impl EventAction<NovaEventWorld> for NextScenarioActionConfig {
    fn action(&self, world: &mut NovaEventWorld, _: &GameEventInfo) {
        world.next_scenario = Some(self.clone());
    }
}

#[derive(Clone, Debug)]
pub struct ObjectiveActionConfig {
    pub id: String,
    pub message: String,
}

impl ObjectiveActionConfig {
    pub fn new(id: &str, message: &str) -> Self {
        Self {
            id: id.to_string(),
            message: message.to_string(),
        }
    }
}

impl EventAction<NovaEventWorld> for ObjectiveActionConfig {
    fn action(&self, world: &mut NovaEventWorld, _: &GameEventInfo) {
        world.objectives.push(self.clone());
    }
}

#[derive(Clone, Debug)]
pub struct ObjectiveCompleteActionConfig {
    pub id: String,
}

impl EventAction<NovaEventWorld> for ObjectiveCompleteActionConfig {
    fn action(&self, world: &mut NovaEventWorld, _: &GameEventInfo) {
        world.objectives.retain(|obj| obj.id != self.id);
    }
}

// TODO: make this into a struct with base options like health, id, name, etc.
// and then specifi Kind which is the asteroid or spaceship or whatever else I add
#[derive(Clone, Debug)]
pub enum ScenarioObjectConfig {
    Asteroid(AsteroidConfig),
    Spaceship(SpaceshipConfig),
}

impl EventAction<NovaEventWorld> for ScenarioObjectConfig {
    fn action(&self, world: &mut NovaEventWorld, info: &GameEventInfo) {
        match self {
            ScenarioObjectConfig::Asteroid(config) => config.action(world, info),
            ScenarioObjectConfig::Spaceship(config) => config.action(world, info),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AsteroidConfig {
    pub id: String,
    pub name: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub radius: f32,
    pub texture: Handle<Image>,
    pub health: f32,
}

impl EventAction<NovaEventWorld> for AsteroidConfig {
    fn action(&self, world: &mut NovaEventWorld, _info: &GameEventInfo) {
        let config = self.clone();

        world.push_command(move |commands| {
            commands.spawn((
                ScenarioScopedMarker,
                Name::new(config.name.clone()),
                EntityId::new(config.id.clone()),
                EntityTypeName::new("asteroid"),
                Transform::from_translation(config.position).with_rotation(config.rotation),
                RigidBody::Dynamic,
                Health::new(config.health),
                ExplodableEntityMarker,
                Visibility::Visible,
                asteroid_game_object(AsteroidConfig1 {
                    radius: config.radius,
                    texture: config.texture.clone(),
                }),
            ));
        });
    }
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
pub struct SpaceshipSectionConfig {
    pub position: Vec3,
    pub rotation: Quat,
    // NOTE: Maybe in the future this will be a Handle and in the .cfg file it will be represented
    // by an ID.
    pub config: SectionConfig,
}

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

impl EventAction<NovaEventWorld> for SpaceshipConfig {
    fn action(&self, world: &mut NovaEventWorld, _info: &GameEventInfo) {
        let config = self.clone();

        world.push_command(move |commands| {
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
                                section_entity.insert(thruster_section(thruster_config.clone()));

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
        });
    }
}
