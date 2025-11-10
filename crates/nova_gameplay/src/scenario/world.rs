use std::collections::VecDeque;

use bevy::{ecs::world::CommandQueue, platform::collections::HashMap, prelude::*};
use bevy_common_systems::prelude::EventWorld;

use crate::prelude::{
    GameObjectivesHud, LoadScenarioById, NextScenarioActionConfig, ObjectiveActionConfig,
    VariableLiteral,
};

#[derive(Resource, Default)]
pub struct NovaEventWorld {
    pub queued_commands: VecDeque<Box<dyn FnOnce(&mut Commands) + Send + Sync>>,
    pub objectives: Vec<ObjectiveActionConfig>,
    pub variables: HashMap<String, VariableLiteral>,
    pub next_scenario: Option<NextScenarioActionConfig>,
}

impl EventWorld for NovaEventWorld {
    fn world_to_state_system(_world: &mut World) {}

    fn state_to_world_system(world: &mut World) {
        // Copy the objectives to the bevy world
        let objectives = &world.resource::<Self>().objectives.clone();
        world.resource_mut::<GameObjectivesHud>().objectives.clear();
        world
            .resource_mut::<GameObjectivesHud>()
            .objectives
            .extend(objectives.iter().cloned());

        // Log variables
        debug!("# Current Variables:");
        for (key, value) in &world.resource::<Self>().variables {
            debug!("Variable: {} = {:?}", key, value);
        }

        // If the next scenario is set, switch
        if let Some(next_scenario) = &world.resource::<Self>().next_scenario {
            if !next_scenario.linger {
                world.trigger(LoadScenarioById(next_scenario.scenario_id.clone()));
                world.resource_mut::<Self>().clear();
            }
        }

        // Apply all the commands in the queue
        let mut event_world = world.resource_mut::<NovaEventWorld>();
        if !event_world.queued_commands.is_empty() {
            let queued_commands = std::mem::take(&mut event_world.queued_commands);

            let mut queue = CommandQueue::default();
            let mut commands = Commands::new(&mut queue, world);

            for cmd in queued_commands.into_iter() {
                cmd(&mut commands);
            }

            queue.apply(world);
        }
    }
}

impl NovaEventWorld {
    pub fn clear(&mut self) {
        self.queued_commands.clear();
        self.objectives.clear();
        self.variables.clear();
        self.next_scenario = None;
    }

    pub fn push_command<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Commands) + Send + Sync + 'static,
    {
        self.queued_commands.push_back(Box::new(f));
    }
}
