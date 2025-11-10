pub mod actions;
pub mod events;
pub mod filters;
pub mod loader;
pub mod objects;
pub mod variables;
pub mod world;

pub mod prelude {
    pub use super::{
        actions::prelude::*, events::prelude::*, filters::prelude::*, loader::prelude::*,
        objects::prelude::*, variables::prelude::*, world::NovaEventWorld, EntityId,
        EntityTypeName, NovaScenarioPlugin,
    };
}

use bevy::prelude::*;
use bevy_common_systems::prelude::*;

#[derive(Component, Debug, Clone, Default, Deref, DerefMut, Reflect)]
pub struct EntityId(pub String);

impl EntityId {
    pub fn new<S: Into<String>>(s: S) -> Self {
        EntityId(s.into())
    }
}

#[derive(Component, Debug, Clone, Default, Deref, DerefMut, Reflect)]
pub struct EntityTypeName(pub String);

impl EntityTypeName {
    pub fn new<S: Into<String>>(s: S) -> Self {
        EntityTypeName(s.into())
    }
}

/// A plugin that handles Game Events.
pub struct NovaScenarioPlugin {
    pub render: bool,
}

impl Plugin for NovaScenarioPlugin {
    fn build(&self, app: &mut App) {
        debug!("NovaEventsPlugin: build");

        app.add_plugins(GameEventsPlugin::<world::NovaEventWorld>::default());
        app.add_plugins(loader::ScenarioLoaderPlugin);
        app.add_plugins(objects::ScenarioObjectsPlugin {
            render: self.render,
        });
    }
}
