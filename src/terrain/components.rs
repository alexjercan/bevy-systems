use crate::noise::map::NoiseInput;
use bevy::prelude::*;
use hexx::*;

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct HexCoord(pub Hex);

impl From<Hex> for HexCoord {
    fn from(hex: Hex) -> Self {
        Self(hex)
    }
}

pub type HexTileKind = i32;
pub type HexFeatureKind = i32;

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct HexTile(pub HexTileKind);

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct HexFeature(pub HexFeatureKind);

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct HexNoiseHeight(pub f64);

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct HexNoiseTemperature(pub f64);

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct HexNoiseHumidity(pub f64);

// Maybe I can implement these in the noise::map module instead?

impl NoiseInput for HexCoord {
    type Query = (&'static HexCoord,);

    fn from_query_item(
        item: bevy::ecs::query::QueryItem<<Self::Query as bevy::ecs::query::QueryData>::ReadOnly>,
    ) -> Self {
        item.0.clone()
    }
}

impl NoiseInput for (HexCoord, HexTile) {
    type Query = (&'static HexCoord, &'static HexTile);

    fn from_query_item(
        item: bevy::ecs::query::QueryItem<<Self::Query as bevy::ecs::query::QueryData>::ReadOnly>,
    ) -> Self {
        (item.0.clone(), item.1.clone())
    }
}
