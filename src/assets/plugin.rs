use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use super::features::{FeatureAsset, FeatureID, FeaturesDynamicAssetCollection};
use super::tiles::{TileAsset, TileID, TilesDynamicAssetCollection};
use crate::GameStates;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetsPluginSet;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<TilesDynamicAssetCollection>::new(&[
            "tile.ron",
        ]))
        .add_plugins(RonAssetPlugin::<FeaturesDynamicAssetCollection>::new(&[
            "feature.ron",
        ]))
        .insert_resource(MapAssets::default())
        .init_asset::<TileAsset>()
        .init_asset::<FeatureAsset>()
        .add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>()
                .register_dynamic_asset_collection::<TilesDynamicAssetCollection>()
                .with_dynamic_assets_file::<TilesDynamicAssetCollection>("terrain.tile.ron")
                .register_dynamic_asset_collection::<FeaturesDynamicAssetCollection>()
                .with_dynamic_assets_file::<FeaturesDynamicAssetCollection>("terrain.feature.ron"),
        )
        .add_systems(Update, handle_map_assets_update.in_set(AssetsPluginSet));
    }
}

#[derive(AssetCollection, Resource, Clone)]
struct GameAssets {
    #[asset(key = "tiles", collection(typed))]
    pub tiles: Vec<Handle<TileAsset>>,
    #[asset(key = "features", collection(typed))]
    pub features: Vec<Handle<FeatureAsset>>,
}

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

#[derive(Resource, Clone, Default)]
pub struct FeatureAssets {}

fn handle_map_assets_update(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    tile_assets: Res<Assets<TileAsset>>,
    feature_assets: Res<Assets<FeatureAsset>>,
) {
    if game_assets.is_changed() {
        info!("Updating MapAssets with game assets");

        let mut map_assets = MapAssets::default();
        map_assets.tiles = game_assets
            .tiles
            .iter()
            .filter_map(|handle| tile_assets.get(handle).cloned())
            .collect();
        map_assets.features = game_assets
            .features
            .iter()
            .filter_map(|handle| feature_assets.get(handle).cloned())
            .collect();
        commands.insert_resource(map_assets.clone());
    }
}
