//! A Bevy plugin that adds various debugging tools.

use bevy::prelude::*;
use bevy_common_systems::prelude::*;

pub mod sections;

pub mod prelude {
    pub use super::{debugdump, DebugPlugin};
}

/// The keycode to toggle debug mode.
pub const DEBUG_TOGGLE_KEYCODE: KeyCode = KeyCode::F11;

/// Resource with debug toggle state.
#[derive(Resource, Default, Clone, Debug, Deref, DerefMut, PartialEq, Eq, Hash)]
pub struct DebugEnabled(pub bool);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DebugSystems;

/// A plugin that adds various debugging tools.
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugEnabled(true));

        app.add_plugins(InspectorDebugPlugin);
        app.add_plugins(WireframeDebugPlugin);
        app.add_plugins(sections::SectionsDebugPlugin);

        app.add_systems(Update, toggle_debug_mode);

        app.configure_sets(
            Update,
            DebugSystems.run_if(resource_equals(DebugEnabled(true))),
        );
        app.configure_sets(
            PostUpdate,
            DebugSystems.run_if(resource_equals(DebugEnabled(true))),
        );
    }
}

fn toggle_debug_mode(mut debug: ResMut<DebugEnabled>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(DEBUG_TOGGLE_KEYCODE) {
        **debug = !**debug;
    }
}

pub fn debugdump(app: &mut App) {
    bevy_mod_debugdump::print_schedule_graph(app, Update);
    // bevy_mod_debugdump::print_schedule_graph(app, PostUpdate);
    // bevy_mod_debugdump::print_schedule_graph(app, FixedUpdate);
}
