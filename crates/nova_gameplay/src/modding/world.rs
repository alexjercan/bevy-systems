use bevy::{platform::collections::HashMap, prelude::*};
use bevy_common_systems::prelude::EventWorld;

use super::{actions::ObjectiveActionConfig, scenario::ScenarioId, variables::VariableLiteral};
use crate::prelude::{GameObjectivesHud, ScenarioLoad};

#[derive(Resource, Default)]
pub struct NovaEventWorld {
    pub objectives: Vec<ObjectiveActionConfig>,
    pub variables: HashMap<String, VariableLiteral>,
    pub next_scenario: Option<ScenarioId>,
}

impl EventWorld for NovaEventWorld {
    fn world_to_state_system(world: &mut World) {
        let mut resource = world.resource_mut::<Self>();

        resource.next_scenario = None;
    }

    fn state_to_world_system(world: &mut World) {
        let objectives = &world.resource::<Self>().objectives.clone();
        world.resource_mut::<GameObjectivesHud>().objectives.clear();
        world
            .resource_mut::<GameObjectivesHud>()
            .objectives
            .extend(objectives.iter().cloned());

        debug!("# Current Variables:");
        for (key, value) in &world.resource::<Self>().variables {
            debug!("Variable: {} = {:?}", key, value);
        }

        if let Some(next_scenario) = &world.resource::<Self>().next_scenario {
            world.trigger(ScenarioLoad(next_scenario.clone()));
        }
    }
}
