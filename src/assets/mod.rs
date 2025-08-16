mod features;
mod plugin;
mod tiles;

pub mod prelude {
    pub use super::features::{FeatureAsset, FeatureID};
    pub use super::plugin::{AssetsPlugin, AssetsPluginSet, MapAssets};
    pub use super::tiles::{TileAsset, TileID};
}
