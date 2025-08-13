use bevy::prelude::*;

use super::overlay::*;
use super::stats::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DebugPluginSet;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StatsPlugin)
            .add_plugins(OverlayPlugin)
            .configure_sets(Update, StatsPluginSet.in_set(DebugPluginSet))
            .configure_sets(Update, OverlayPluginSet.in_set(DebugPluginSet));
    }
}
