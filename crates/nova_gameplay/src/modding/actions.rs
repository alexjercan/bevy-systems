use bevy::prelude::*;
use bevy_common_systems::modding::prelude::*;
use super::variables::prelude::*;

use super::world::NovaEventWorld;

pub mod prelude {
    pub use super::{
        DebugMessageActionConfig, EventActionConfig, ObjectiveActionConfig, VariableSetActionConfig,
    };
}

#[derive(Clone, Debug)]
pub enum EventActionConfig {
    DebugMessage(DebugMessageActionConfig),
    VariableSet(VariableSetActionConfig),
    Objective(ObjectiveActionConfig),
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
