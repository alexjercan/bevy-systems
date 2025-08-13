use super::components::*;
use super::planet::*;
use bevy::prelude::*;
use hexx::*;
use crate::{
    hexmap::map::{HexMapPlugin, HexMapSet},
    noise::map::{NoisePlugin, NoiseSet},
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlanetPluginSet;

pub struct PlanetPlugin {
    seed: u32,
    layout: HexLayout,
    chunk_radius: u32,
    discover_radius: u32,
}

impl PlanetPlugin {
    pub fn new(seed: u32, layout: HexLayout, chunk_radius: u32, discover_radius: u32) -> Self {
        Self {
            seed,
            layout,
            chunk_radius,
            discover_radius,
        }
    }
}

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HexMapPlugin::<HexCoord>::new(
            self.layout.clone(),
            self.chunk_radius,
            self.discover_radius,
        ))
        .add_plugins(NoisePlugin::<HexCoord, HexNoiseHeight, _>::new(
            PlanetHeight::default().with_seed(self.seed),
        ))
        .add_plugins(NoisePlugin::<HexCoord, HexNoiseTemperature, _>::new(
            PlanetTemperature::default().with_seed(self.seed + 1),
        ))
        .add_plugins(NoisePlugin::<HexCoord, HexNoiseHumidity, _>::new(
            PlanetHumidity::default().with_seed(self.seed + 2),
        ))
        // .add_plugins(NoisePlugin::<(HexCoord, HexNoiseHeight), HexFeature, _>::new(
        //     PlanetFeatures::default().with_seed(self.seed + 3),
        // ))
        .configure_sets(Update, HexMapSet.in_set(PlanetPluginSet))
        .configure_sets(Update, NoiseSet.in_set(PlanetPluginSet));
    }
}
