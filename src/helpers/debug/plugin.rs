use bevy::prelude::*;

use super::stats::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DebugPluginSet;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StatsPlugin)
            .configure_sets(Update, StatsPluginSet.in_set(DebugPluginSet));
    }
}
