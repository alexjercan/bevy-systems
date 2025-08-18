use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use super::features::{FeatureAsset, FeatureID, FeaturesDynamicAssetCollection};
use super::tiles::{TileAsset, TileID, TilesDynamicAssetCollection};
use crate::terrain::prelude::MapAssets;
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
