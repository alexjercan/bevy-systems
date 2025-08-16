use bevy::{ecs::system::SystemState, platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;

use super::tiles::TileID;

pub type FeatureID = String;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct FeatureAsset {
    pub id: FeatureID,
    pub name: String,
    variants: Vec<FeatureVariant>,
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

#[derive(serde::Deserialize, Debug, Clone)]
struct FeatureDynamicAsset {
    id: String,
    name: String,
    variants: Vec<FeatureVariantDynamic>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct FeatureVariantDynamic {
    id: String,
    name: String,
    threshold: f64,
    scene: String,
}

#[derive(serde::Deserialize, Debug, Clone, Deref)]
struct FeaturesDynamicAsset(Vec<FeatureDynamicAsset>);

impl DynamicAsset for FeaturesDynamicAsset {
    fn load(&self, asset_server: &AssetServer) -> Vec<UntypedHandle> {
        self.iter()
            .flat_map(|feature| {
                feature
                    .variants
                    .iter()
                    .map(|variant| asset_server.load_untyped(&variant.scene).untyped())
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
                    let variants = feature
                        .variants
                        .iter()
                        .map(|variant| FeatureVariant {
                            id: variant.id.clone(),
                            name: variant.name.clone(),
                            threshold: variant.threshold,
                            scene: asset_server.load(&variant.scene),
                        })
                        .collect();

                    let feature_asset = FeatureAsset {
                        id: feature.id.clone(),
                        name: feature.name.clone(),
                        variants,
                    };

                    terrain.add(feature_asset).untyped()
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
