use bevy::{ecs::system::SystemState, platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;

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

#[derive(serde::Deserialize, Debug, Clone)]
struct TileDynamicAsset {
    id: String,
    name: String,
    generation: TileGenerationDynamic,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct TileGenerationDynamic {
    elevation: Option<Vec2>,
    humidity: Option<Vec2>,
    temperature: Option<Vec2>,
}

#[derive(serde::Deserialize, Debug, Clone, Deref)]
struct TilesDynamicAsset(Vec<TileDynamicAsset>);

impl DynamicAsset for TilesDynamicAsset {
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
                        generation: TileGeneration {
                            elevation: tile.generation.elevation,
                            humidity: tile.generation.humidity,
                            temperature: tile.generation.temperature,
                        },
                    };

                    terrain.add(tile).untyped()
                })
                .collect(),
        ));
    }
}

#[derive(serde::Deserialize, Asset, TypePath)]
pub(super) struct TilesDynamicAssetCollection(HashMap<String, TilesDynamicAsset>);

impl DynamicAssetCollection for TilesDynamicAssetCollection {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(asset.clone()));
        }
    }
}
