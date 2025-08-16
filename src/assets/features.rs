use bevy::{ecs::system::SystemState, platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct FeatureAsset {
    pub id: String,
    pub name: String,
    pub threshold: Vec<f64>,
    pub scene: Vec<Option<Handle<Scene>>>,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct FeatureDynamicAsset {
    id: String,
    name: String,
    threshold: Vec<f64>,
    model: Vec<Option<String>>,
}

#[derive(serde::Deserialize, Debug, Clone, Deref)]
struct FeaturesDynamicAsset(Vec<FeatureDynamicAsset>);

impl DynamicAsset for FeaturesDynamicAsset {
    fn load(&self, asset_server: &AssetServer) -> Vec<UntypedHandle> {
        self.iter()
            .flat_map(|feature| {
                feature.model.iter().filter_map(|model| {
                    model
                        .as_ref()
                        .map(|m| asset_server.load_untyped(m).untyped())
                })
            })
            .collect()
    }

    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
        let mut system_state =
            SystemState::<(ResMut<Assets<FeatureAsset>>, Res<AssetServer>)>::new(world);
        let (mut terrain, asset_server) = system_state.get_mut(world);

        return Ok(DynamicAssetType::Collection(
            self.iter()
                .map(|feature| {
                    let scene = feature
                        .model
                        .iter()
                        .map(|model| model.as_ref().map(|m| asset_server.load(m)))
                        .collect();

                    let tile = FeatureAsset {
                        id: feature.id.clone(),
                        name: feature.name.clone(),
                        threshold: feature.threshold.clone(),
                        scene: scene,
                    };

                    terrain.add(tile).untyped()
                })
                .collect(),
        ));
    }
}

#[derive(serde::Deserialize, Asset, TypePath)]
pub(super) struct FeaturesDynamicAssetCollection(HashMap<String, FeaturesDynamicAsset>);

impl DynamicAssetCollection for FeaturesDynamicAssetCollection {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(asset.clone()));
        }
    }
}
