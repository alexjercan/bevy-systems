use bevy::prelude::*;
use hexx::*;
use crate::noise::map::NoiseInput;

use crate::assets::prelude::FeatureAsset;

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct HexCoord(pub Hex);

impl From<Hex> for HexCoord {
    fn from(hex: Hex) -> Self {
        Self(hex)
    }
}

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct HexFeature(pub FeatureAsset);

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct HexNoiseHeight(pub f64);

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct HexNoiseTemperature(pub f64);

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct HexNoiseHumidity(pub f64);

impl NoiseInput for HexCoord {
    type Query = (&'static HexCoord,);

    fn from_query_item(
        item: bevy::ecs::query::QueryItem<<Self::Query as bevy::ecs::query::QueryData>::ReadOnly>,
    ) -> Self {
        item.0.clone()
    }
}

impl NoiseInput for (HexCoord, HexNoiseHeight) {
    type Query = (&'static HexCoord, &'static HexNoiseHeight);

    fn from_query_item(
        item: bevy::ecs::query::QueryItem<<Self::Query as bevy::ecs::query::QueryData>::ReadOnly>,
    ) -> Self {
        (item.0.clone(), item.1.clone())
    }
}
