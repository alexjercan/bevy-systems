use bevy::{platform::collections::HashMap, prelude::*};
use bevy_common_systems::prelude::EventWorld;

use crate::prelude::{
    GameObjectivesHud, NextScenarioActionConfig, ObjectiveActionConfig, ScenarioLoad,
    VariableLiteral,
};

#[derive(Resource, Default)]
pub struct NovaEventWorld {
    pub objectives: Vec<ObjectiveActionConfig>,
    pub variables: HashMap<String, VariableLiteral>,
    pub next_scenario: Option<NextScenarioActionConfig>,
}

impl EventWorld for NovaEventWorld {
    fn world_to_state_system(_world: &mut World) {}

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
            if !next_scenario.linger {
                world.trigger(ScenarioLoad(next_scenario.scenario_id.clone()));
                world.resource_mut::<Self>().clear();
            }
        }
    }
}

impl NovaEventWorld {
    pub fn clear(&mut self) {
        self.objectives.clear();
        self.variables.clear();
        self.next_scenario = None;
    }
}
