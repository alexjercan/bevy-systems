use bevy::{ecs::system::SystemState, platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use crate::GameStates;

pub mod prelude {
    pub use super::{AssetsPlugin, AssetsPluginSet, GameAssets, TileAsset};
}

#[derive(AssetCollection, Resource, Clone)]
pub struct GameAssets {
    #[asset(key = "terrain", collection(typed))]
    pub terrain: Vec<Handle<TileAsset>>,
}

impl GameAssets {
    pub fn terrain_index(&self, terrain: &Assets<TileAsset>, elevation: f32, humidity: f32, temperature: f32) -> Option<usize> {
        self.terrain.iter().position(|tile| {
            let Some(tile) = terrain.get(tile) else {
                return false;
            };

            tile.elevation.as_ref().map_or(true, |e| {
                e.x <= elevation && elevation <= e.y
            }) && tile.humidity.as_ref().map_or(true, |h| {
                h.x <= humidity && humidity <= h.y
            }) && tile.temperature.as_ref().map_or(true, |t| {
                t.x <= temperature && temperature <= t.y
            })
        })
    }
}

#[derive(Asset, TypePath, Debug)]
pub struct TileAsset {
    pub id: String,
    pub name: String,
    pub elevation: Option<Vec2>,
    pub humidity: Option<Vec2>,
    pub temperature: Option<Vec2>,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct TileDynamicAsset {
    id: String,
    name: String,
    elevation: Option<Vec2>,
    humidity: Option<Vec2>,
    temperature: Option<Vec2>,
}

#[derive(serde::Deserialize, Debug, Clone, Deref)]
struct TerrainDynamicAsset(Vec<TileDynamicAsset>);

impl DynamicAsset for TerrainDynamicAsset {
    fn load(&self, _: &AssetServer) -> Vec<UntypedHandle> {
        vec![]
    }

    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
        let mut system_state =
            SystemState::<(ResMut<Assets<TileAsset>>, Res<AssetServer>)>::new(world);
        let (mut terrain, _) = system_state.get_mut(world);

        return Ok(DynamicAssetType::Collection(
            self.iter()
                .map(|tile| {
                    let tile = TileAsset {
                        id: tile.id.clone(),
                        name: tile.name.clone(),
                        elevation: tile.elevation,
                        humidity: tile.humidity,
                        temperature: tile.temperature,
                    };

                    terrain.add(tile).untyped()
                })
                .collect(),
        ));
    }
}

#[derive(serde::Deserialize, Asset, TypePath)]
struct TerrainDynamicAssetCollection(HashMap<String, TerrainDynamicAsset>);

impl DynamicAssetCollection for TerrainDynamicAssetCollection {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(asset.clone()));
        }
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetsPluginSet;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<TerrainDynamicAssetCollection>::new(&[
            "terrain.ron",
        ]))
        .init_asset::<TileAsset>()
        .add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>()
                .register_dynamic_asset_collection::<TerrainDynamicAssetCollection>()
                .with_dynamic_assets_file::<TerrainDynamicAssetCollection>(
                    "tiles.terrain.ron",
                ),
        );
    }
}
