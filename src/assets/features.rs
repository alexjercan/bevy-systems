use bevy::{ecs::system::SystemState, platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct FeatureAsset {
    pub id: String,
    pub name: String,
    pub frequency: f64,
    pub threshold: f64,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct FeatureDynamicAsset {
    id: String,
    name: String,
    frequency: f64,
    threshold: f64,
}

#[derive(serde::Deserialize, Debug, Clone, Deref)]
struct FeaturesDynamicAsset(Vec<FeatureDynamicAsset>);

impl DynamicAsset for FeaturesDynamicAsset {
    fn load(&self, _: &AssetServer) -> Vec<UntypedHandle> {
        vec![]
    }

    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
        let mut system_state =
            SystemState::<(ResMut<Assets<FeatureAsset>>, Res<AssetServer>)>::new(world);
        let (mut terrain, _) = system_state.get_mut(world);

        return Ok(DynamicAssetType::Collection(
            self.iter()
                .map(|feature| {
                    let tile = FeatureAsset {
                        id: feature.id.clone(),
                        name: feature.name.clone(),
                        frequency: feature.frequency,
                        threshold: feature.threshold,
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

