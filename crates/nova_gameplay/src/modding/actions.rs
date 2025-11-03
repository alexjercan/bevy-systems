use bevy::prelude::*;
use bevy_common_systems::modding::prelude::*;
use super::world::NovaEventWorld;

pub mod prelude {
    pub use super::{EventActionConfig, PrintMessageActionConfig};
}

#[derive(Clone, Debug)]
pub enum EventActionConfig {
    PrintMessage(PrintMessageActionConfig),
}

impl EventAction<NovaEventWorld> for EventActionConfig {
    fn action(&self, world: &mut NovaEventWorld, info: &GameEventInfo) {
        match self {
            EventActionConfig::PrintMessage(config) => {
                config.action(world, info);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct PrintMessageActionConfig {
    pub message: String,
}

impl EventAction<NovaEventWorld> for PrintMessageActionConfig {
    fn action(&self, _: &mut NovaEventWorld, _: &GameEventInfo) {
        println!("Event Action Message: {}", self.message);
    }
}
