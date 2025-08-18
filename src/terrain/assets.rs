use bevy::prelude::*;

use crate::assets::prelude::*;

#[derive(Resource, Clone, Default, Debug)]
pub struct MapAssets {
    pub tiles: Vec<TileAsset>,
    pub features: Vec<FeatureAsset>,
}

impl MapAssets {
    pub fn terrain_index(&self, elevation: f32, humidity: f32, temperature: f32) -> Option<TileID> {
        self.tiles
            .iter()
            .find(|tile| {
                let generation = &tile.generation;

                generation
                    .elevation
                    .as_ref()
                    .map_or(true, |e| e.x <= elevation && elevation <= e.y)
                    && generation
                        .humidity
                        .as_ref()
                        .map_or(true, |h| h.x <= humidity && humidity <= h.y)
                    && generation
                        .temperature
                        .as_ref()
                        .map_or(true, |t| t.x <= temperature && temperature <= t.y)
            })
            .map(|tile| tile.id.clone())
    }

    pub fn get_tile(&self, id: &TileID) -> Option<&TileAsset> {
        self.tiles.iter().find(|tile| &tile.id == id)
    }

    pub fn get_tile_index(&self, id: &TileID) -> Option<usize> {
        self.tiles.iter().position(|tile| &tile.id == id)
    }

    pub fn get_feature(&self, id: &FeatureID) -> Option<&FeatureAsset> {
        self.features.iter().find(|feature| &feature.id == id)
    }
}
