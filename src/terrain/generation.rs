use super::{assets::*, components::*, planet::*};
use crate::helpers::prelude::*;
use bevy::prelude::*;
use hexx::*;
use itertools::Itertools;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GenerationPluginSet;

pub struct GenerationPlugin {
    seed: u32,
    layout: HexLayout,
    chunk_radius: u32,
    discover_radius: u32,
}

impl GenerationPlugin {
    pub fn new(seed: u32, layout: HexLayout, chunk_radius: u32, discover_radius: u32) -> Self {
        Self {
            seed,
            layout,
            chunk_radius,
            discover_radius,
        }
    }
}

impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HexMapPlugin::<HexCoord>::new(
            self.layout.clone(),
            self.chunk_radius,
            self.discover_radius,
        ))
        .add_plugins(ChunkMapPlugin::<HexCoord, HexNoiseHeight, _>::new(
            PlanetHeight::default().with_seed(self.seed),
        ))
        .add_plugins(ChunkMapPlugin::<HexCoord, HexNoiseTemperature, _>::new(
            PlanetTemperature::default().with_seed(self.seed + 1),
        ))
        .add_plugins(ChunkMapPlugin::<HexCoord, HexNoiseHumidity, _>::new(
            PlanetHumidity::default().with_seed(self.seed + 2),
        ))
        .add_plugins(ChunkMapPlugin::<(HexCoord, HexTile), HexFeature, _>::new(
            PlanetFeatures::default().with_seed(self.seed + 3),
        ))
        .configure_sets(Update, HexMapSet.in_set(GenerationPluginSet))
        .configure_sets(Update, ChunkMapPluginSet.in_set(GenerationPluginSet))
        .add_systems(Update, handle_chunk.in_set(GenerationPluginSet))
        .add_systems(Update, handle_features.in_set(GenerationPluginSet));
    }
}

fn handle_features(assets: Res<TerrainAssets>, mut planet: ResMut<PlanetFeatures>) {
    if assets.is_changed() {
        debug!("Updating planet features with new assets");
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
    assets: Res<TerrainAssets>,
) {
    if q_hex.is_empty() {
        return;
    }
    debug!("Handling hex tile for {} hexes", q_hex.iter().len());

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
