use bevy::{platform::collections::HashMap, prelude::*};
use bevy_common_systems::prelude::EventWorld;

use crate::prelude::ScenarioLoad;

use super::{actions::ObjectiveActionConfig, variables::VariableLiteral, scenario::ScenarioId};

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
        println!("# Current Objectives:");
        for objective in &world.resource::<Self>().objectives {
            println!("Objective: {} - {}", objective.id, objective.message);
        }
        println!("# Current Variables:");
        for (key, value) in &world.resource::<Self>().variables {
            println!("Variable: {} = {:?}", key, value);
        }

        if let Some(next_scenario) = &world.resource::<Self>().next_scenario {
            world.trigger(ScenarioLoad(next_scenario.clone()));
        }
    }
}
