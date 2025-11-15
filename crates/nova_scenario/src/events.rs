use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use nova_events::prelude::*;

use crate::prelude::*;

pub mod prelude {
    pub use super::EventConfig;
}

#[derive(Debug, Clone, Copy, Reflect)]
pub enum EventConfig {
    OnStart,
    OnDestroyed,
    OnUpdate,
}

impl From<EventConfig> for EventHandler<NovaEventWorld> {
    fn from(value: EventConfig) -> Self {
        match value {
            EventConfig::OnStart => EventHandler::new::<OnStartEvent>(),
            EventConfig::OnDestroyed => EventHandler::new::<OnDestroyedEvent>(),
            EventConfig::OnUpdate => EventHandler::new::<OnUpdateEvent>(),
        }
    }
}
