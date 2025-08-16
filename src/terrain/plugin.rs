use super::components::*;
use super::planet::*;
use crate::assets::prelude::*;
use crate::{
    noise::map::{NoisePlugin, NoiseSet},
    tilemap::hexmap::{HexMapPlugin, HexMapSet},
};
use bevy::prelude::*;
use hexx::*;
use itertools::Itertools;

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
        .add_plugins(NoisePlugin::<(HexCoord, HexTile), HexFeature, _>::new(
            PlanetFeatures::default().with_seed(self.seed + 3),
        ))
        .configure_sets(Update, HexMapSet.in_set(PlanetPluginSet))
        .configure_sets(Update, NoiseSet.in_set(PlanetPluginSet))
        .add_systems(Update, handle_chunk.in_set(PlanetPluginSet))
        .add_systems(Update, handle_features.in_set(PlanetPluginSet));
    }
}

fn handle_features(assets: Res<MapAssets>, mut planet: ResMut<PlanetFeatures>) {
    if assets.is_changed() {
        *planet = planet.clone().with_map(assets.clone());
    }
}

fn handle_chunk(
    mut commands: Commands,
    q_hex: Query<
        (
            Entity,
            &HexNoiseHeight,
            &HexNoiseHumidity,
            &HexNoiseTemperature,
            &ChildOf,
        ),
        (With<HexCoord>, Without<HexTile>),
    >,
    assets: Res<MapAssets>,
) {
    for (&chunk_entity, chunk) in q_hex
        .iter()
        .chunk_by(|(_, _, _, _, ChildOf(e))| e)
        .into_iter()
    {
        for (entity, height, humidity, temperature, _) in chunk {
            let height = **height as f32;
            let humidity = **humidity as f32;
            let temperature = **temperature as f32;

            let kind = assets.terrain_index(height, humidity, temperature);

            match kind {
                Some(kind) => {
                    commands.entity(chunk_entity).add_child(entity);
                    commands.entity(entity).insert(HexTile(kind));
                }
                None => {
                    warn!(
                        "No tile found for height: {}, humidity: {}, temperature: {}",
                        height, humidity, temperature
                    );
                }
            }
        }
    }
}
