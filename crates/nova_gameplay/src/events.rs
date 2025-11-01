pub mod prelude {
    pub use super::{
        EntityFilter, InsertComponentAction, NovaEventWorld, NovaEventsPlugin, OnDestroyedEvent,
        OnDestroyedEventInfo,
    };
}

use std::collections::VecDeque;

use bevy::{ecs::world::CommandQueue, prelude::*};
use bevy_common_systems::prelude::*;

/// A plugin that handles Game Events.
pub struct NovaEventsPlugin;

impl Plugin for NovaEventsPlugin {
    fn build(&self, app: &mut App) {
        debug!("NovaEventsPlugin: build");

        app.add_plugins(GameEventsPlugin::<NovaEventWorld>::default());
    }
}

#[derive(Resource, Default)]
pub struct NovaEventWorld {
    queued_commands: VecDeque<Box<dyn FnOnce(&mut Commands) + Send + Sync>>,
}

impl NovaEventWorld {
    pub fn push_command<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Commands) + Send + Sync + 'static,
    {
        self.queued_commands.push_back(Box::new(f));
    }
}

impl EventWorld for NovaEventWorld {
    fn world_to_state_system(_world: &mut World) {}

    fn state_to_world_system(world: &mut World) {
        let mut event_world = world.resource_mut::<NovaEventWorld>();
        let queued_commands = std::mem::take(&mut event_world.queued_commands);

        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, world);

        for cmd in queued_commands.into_iter() {
            cmd(&mut commands);
        }

        queue.apply(world);
    }
}

#[derive(Debug, Clone, EventKind, Reflect)]
#[event_name("ondestroyed")]
#[event_info(OnDestroyedEventInfo)]
pub struct OnDestroyedEvent;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Reflect)]
pub struct OnDestroyedEventInfo {
    pub entity: Entity,
}

impl Default for OnDestroyedEventInfo {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Debug, Clone, Deref, DerefMut, Reflect)]
pub struct InsertComponentAction<C: Component + std::fmt::Debug + Clone + Reflect>(pub C);

impl<C> EventAction<NovaEventWorld> for InsertComponentAction<C>
where
    C: Component + std::fmt::Debug + Clone + Reflect,
{
    fn action(&self, world: &mut NovaEventWorld, info: &GameEventInfo) {
        let Some(data) = &info.data else {
            warn!("InsertComponentAction: no data in event info");
            return;
        };

        let Some(value) = data.get("entity").and_then(|v| v.as_u64()) else {
            warn!("InsertComponentAction: no entity in event info data");
            return;
        };

        let entity = Entity::from_bits(value);

        let component = self.0.clone();
        world.push_command(move |commands| {
            commands.entity(entity).insert(component);
        });
    }
}

#[derive(Debug, Clone, Default, Reflect)]
pub struct EntityFilter {
    pub entity: Option<Entity>,
}

impl EntityFilter {
    pub fn with_id(mut self, entity: Entity) -> Self {
        self.entity = Some(entity);
        self
    }
}

impl EventFilter<NovaEventWorld> for EntityFilter {
    fn filter(&self, _: &NovaEventWorld, info: &GameEventInfo) -> bool {
        let Some(data) = &info.data else {
            return false;
        };

        let Some(value) = data.get("entity").and_then(|v| v.as_u64()) else {
            return false;
        };

        let Some(entity) = self.entity else {
            return true;
        };

        value == entity.to_bits()
    }
}
