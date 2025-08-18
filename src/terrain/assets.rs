use bevy::prelude::*;

#[derive(Resource, Clone, Default, Debug)]
pub struct TerrainAssets {
    pub tiles: Vec<TileAsset>,
    pub features: Vec<FeatureAsset>,
}

impl TerrainAssets {
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

    pub fn get_tile_index(&self, id: &TileID) -> Option<usize> {
        self.tiles.iter().position(|tile| &tile.id == id)
    }

    pub fn get_feature(&self, id: &FeatureID) -> Option<&FeatureAsset> {
        self.features.iter().find(|feature| &feature.id == id)
    }
}

pub type FeatureID = String;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct FeatureAsset {
    pub id: FeatureID,
    pub name: String,
    pub variants: Vec<FeatureVariant>,
}

impl FeatureAsset {
    pub fn get_variant(&self, id: &TileID) -> Option<&FeatureVariant> {
        self.variants.iter().find(|variant| &variant.id == id)
    }
}

#[derive(Debug, Clone)]
pub struct FeatureVariant {
    pub id: TileID,
    pub name: String,
    pub threshold: f64,
    pub scene: Handle<Scene>,
}

pub type TileID = String;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct TileAsset {
    pub id: TileID,
    pub name: String,
    pub generation: TileGeneration,
}

#[derive(Debug, Clone)]
pub struct TileGeneration {
    pub elevation: Option<Vec2>,
    pub humidity: Option<Vec2>,
    pub temperature: Option<Vec2>,
}
