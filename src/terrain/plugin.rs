use super::{generation::*, render::*};
use bevy::prelude::*;
use hexx::HexLayout;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TerrainPluginSet;

pub struct TerrainPlugin {
    seed: u32,
    layout: HexLayout,
    chunk_radius: u32,
    discover_radius: u32,
    max_height: f32,
}

impl TerrainPlugin {
    pub fn new(
        seed: u32,
        layout: HexLayout,
        chunk_radius: u32,
        discover_radius: u32,
        max_height: f32,
    ) -> Self {
        Self {
            seed,
            layout,
            chunk_radius,
            discover_radius,
            max_height,
        }
    }
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GenerationPlugin::new(
            self.seed,
            self.layout.clone(),
            self.chunk_radius,
            self.discover_radius,
        ))
        .add_plugins(RenderPlugin::new(
            self.layout.clone(),
            self.chunk_radius,
            self.max_height,
        ))
        .configure_sets(Update, GenerationPluginSet.in_set(TerrainPluginSet))
        .configure_sets(Update, RenderPluginSet.in_set(TerrainPluginSet));
    }
}
