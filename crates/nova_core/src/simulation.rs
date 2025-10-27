//! The simulation plugin. This plugin should contain all the gameplay related logic.

use bevy::prelude::*;

pub mod prelude {}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationPluginSet;

pub struct SimulationPlugin;
