use bevy::prelude::*;

use super::overlay::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DebugPluginSet;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(systems::debug::DebugPlugin)
            .add_plugins(OverlayPlugin)
            .configure_sets(Update, systems::debug::DebugSet.in_set(DebugPluginSet))
            .configure_sets(Update, OverlayPluginSet.in_set(DebugPluginSet));
    }
}
