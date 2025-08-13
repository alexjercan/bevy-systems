use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use crate::GameStates;

mod features;
mod tiles;

pub mod prelude {
    pub use super::{AssetsPlugin, AssetsPluginSet, GameAssets};
    pub use super::tiles::TileAsset;
    pub use super::features::FeatureAsset;
}

#[derive(AssetCollection, Resource, Clone)]
pub struct GameAssets {
    #[asset(key = "tiles", collection(typed))]
    pub tiles: Vec<Handle<tiles::TileAsset>>,
    #[asset(key = "features", collection(typed))]
    pub features: Vec<Handle<features::FeatureAsset>>,
}

impl GameAssets {
    pub fn terrain_index(
        &self,
        terrain: &Assets<tiles::TileAsset>,
        elevation: f32,
        humidity: f32,
        temperature: f32,
    ) -> Option<usize> {
        self.tiles.iter().position(|tile| {
            let Some(tile) = terrain.get(tile) else {
                return false;
            };

            tile.elevation
                .as_ref()
                .map_or(true, |e| e.x <= elevation && elevation <= e.y)
                && tile
                    .humidity
                    .as_ref()
                    .map_or(true, |h| h.x <= humidity && humidity <= h.y)
                && tile
                    .temperature
                    .as_ref()
                    .map_or(true, |t| t.x <= temperature && temperature <= t.y)
        })
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetsPluginSet;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<tiles::TilesDynamicAssetCollection>::new(
            &["tile.ron"],
        ))
        .add_plugins(
            RonAssetPlugin::<features::FeaturesDynamicAssetCollection>::new(&["feature.ron"]),
        )
        .init_asset::<tiles::TileAsset>()
        .init_asset::<features::FeatureAsset>()
        .add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>()
                .register_dynamic_asset_collection::<tiles::TilesDynamicAssetCollection>()
                .with_dynamic_assets_file::<tiles::TilesDynamicAssetCollection>("terrain.tile.ron")
                .register_dynamic_asset_collection::<features::FeaturesDynamicAssetCollection>()
                .with_dynamic_assets_file::<features::FeaturesDynamicAssetCollection>(
                    "terrain.feature.ron",
                ),
        );
    }
}
