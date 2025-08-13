use super::components::*;
use super::planet::*;
use bevy::prelude::*;
use hexx::*;
use systems::{
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
        .add_plugins(NoisePlugin::<f64, f64, 3, _, HexCoord, HexNoiseHeight>::new(
            PlanetHeight::default().with_seed(self.seed),
        ))
        .add_plugins(NoisePlugin::<f64, f64, 3, _, HexCoord, HexNoiseTemperature>::new(
            PlanetTemperature::default().with_seed(self.seed + 1),
        ))
        .add_plugins(NoisePlugin::<f64, f64, 3, _, HexCoord, HexNoiseHumidity>::new(
            PlanetHumidity::default().with_seed(self.seed + 2),
        ))
        .configure_sets(Update, HexMapSet.in_set(PlanetPluginSet))
        .configure_sets(Update, NoiseSet.in_set(PlanetPluginSet));
    }
}
