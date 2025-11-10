use bevy::prelude::*;
use bevy_common_systems::modding::prelude::*;

use crate::prelude::*;

pub mod prelude {
    pub use super::{
        ConditionalFilterConfig, EntityFilterConfig, EventFilterConfig, ExpressionFilterConfig,
    };
}

#[derive(Clone, Debug)]
pub enum EventFilterConfig {
    Entity(EntityFilterConfig),
    Conditional(ConditionalFilterConfig),
    Expression(ExpressionFilterConfig),
}

impl EventFilter<NovaEventWorld> for EventFilterConfig {
    fn filter(&self, world: &NovaEventWorld, info: &GameEventInfo) -> bool {
        match self {
            EventFilterConfig::Entity(config) => config.filter(world, info),
            EventFilterConfig::Conditional(config) => config.filter(world, info),
            EventFilterConfig::Expression(config) => config.filter(world, info),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EntityFilterConfig {
    pub id: Option<String>,
    pub type_name: Option<String>,
}

impl EventFilter<NovaEventWorld> for EntityFilterConfig {
    fn filter(&self, _: &NovaEventWorld, info: &GameEventInfo) -> bool {
        let Some(data) = &info.data else {
            return false;
        };

        let Some(id_value) = data.get("id").and_then(|v| v.as_str()) else {
            return false;
        };

        let Some(type_name_value) = data.get("type_name").and_then(|v| v.as_str()) else {
            return false;
        };

        let mut result = true;
        match &self.id {
            Some(id) => result &= id_value == id,
            None => result &= true,
        }

        match &self.type_name {
            Some(type_name) => result &= type_name_value == type_name,
            None => result &= true,
        }

        result
    }
}

#[derive(Clone, Debug)]
pub enum ConditionalFilterConfig {
    Not(Box<EventFilterConfig>),
    Or(Box<EventFilterConfig>, Box<EventFilterConfig>),
    And(Box<EventFilterConfig>, Box<EventFilterConfig>),
}

impl ConditionalFilterConfig {
    pub fn not(inner: EventFilterConfig) -> Self {
        ConditionalFilterConfig::Not(Box::new(inner))
    }

    pub fn or(left: EventFilterConfig, right: EventFilterConfig) -> Self {
        ConditionalFilterConfig::Or(Box::new(left), Box::new(right))
    }

    pub fn and(left: EventFilterConfig, right: EventFilterConfig) -> Self {
        ConditionalFilterConfig::And(Box::new(left), Box::new(right))
    }
}

impl EventFilter<NovaEventWorld> for ConditionalFilterConfig {
    fn filter(&self, world: &NovaEventWorld, info: &GameEventInfo) -> bool {
        match self {
            ConditionalFilterConfig::Not(inner) => !inner.filter(world, info),
            ConditionalFilterConfig::Or(left, right) => {
                left.filter(world, info) || right.filter(world, info)
            }
            ConditionalFilterConfig::And(left, right) => {
                left.filter(world, info) && right.filter(world, info)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExpressionFilterConfig(pub VariableConditionNode);

impl EventFilter<NovaEventWorld> for ExpressionFilterConfig {
    fn filter(&self, world: &NovaEventWorld, _: &GameEventInfo) -> bool {
        match self.0.evaluate(world) {
            Ok(result) => result,
            Err(e) => {
                // TODO: Proper error handling
                error!(
                    "VariableFilterConfig: failed to evaluate condition: {:?}",
                    e
                );
                false
            }
        }
    }
}
