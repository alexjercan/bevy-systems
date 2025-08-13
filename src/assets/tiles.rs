use bevy::{ecs::system::SystemState, platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;

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
pub(super) struct TilesDynamicAssetCollection(HashMap<String, TilesDynamicAsset>);

impl DynamicAssetCollection for TilesDynamicAssetCollection {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(asset.clone()));
        }
    }
}
