pub mod actions;
pub mod events;
pub mod filters;
pub mod scenario;
pub mod variables;
pub mod world;

pub mod prelude {
    pub use super::{
        actions::prelude::*, events::prelude::*, filters::prelude::*, scenario::prelude::*,
        variables::prelude::*, world::NovaEventWorld, NovaEventsPlugin,
    };
}

use bevy::prelude::*;
use bevy_common_systems::prelude::*;

/// A plugin that handles Game Events.
pub struct NovaEventsPlugin;

impl Plugin for NovaEventsPlugin {
    fn build(&self, app: &mut App) {
        debug!("NovaEventsPlugin: build");

        app.add_plugins(GameEventsPlugin::<world::NovaEventWorld>::default());
        app.add_plugins(scenario::ScenarioLoaderPlugin);
    }
}
