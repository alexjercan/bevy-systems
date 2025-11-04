use bevy::{platform::collections::HashMap, prelude::*};
use bevy_common_systems::prelude::EventWorld;

use super::{actions::ObjectiveActionConfig, variables::VariableLiteral};

#[derive(Resource, Default)]
pub struct NovaEventWorld {
    pub objectives: Vec<ObjectiveActionConfig>,
    pub variables: HashMap<String, VariableLiteral>,
}

impl EventWorld for NovaEventWorld {
    fn world_to_state_system(_world: &mut World) {}

    fn state_to_world_system(world: &mut World) {
        println!("# Current Objectives:");
        for objective in &world.resource::<NovaEventWorld>().objectives {
            println!("Objective: {} - {}", objective.id, objective.message);
        }
        println!("# Current Variables:");
        for (key, value) in &world.resource::<NovaEventWorld>().variables {
            println!("Variable: {} = {:?}", key, value);
        }
    }
}
