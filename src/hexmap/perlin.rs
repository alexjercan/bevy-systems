//! TODO: Document this module

use bevy::{platform::collections::HashMap, prelude::*};
use hexx::*;
use noise::{NoiseFn, Perlin};

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct HexNoise(pub f32);

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct HexCoord(pub Hex);

#[derive(Component, Clone, Debug)]
pub struct HexProbe;

#[derive(Debug, Clone)]
struct Chunk {
    center: Hex,
    hexes: Vec<(Hex, f32)>,
}

#[derive(Resource, Debug, Clone)]
struct HexMapGenerator {
    layout: HexLayout,
    perlin: Perlin,
    chunk_radius: u32,
}

impl HexMapGenerator {
    fn chunk(&self, center: Hex) -> Chunk {
        let hexes = shapes::hexagon(Hex::ZERO, self.chunk_radius)
            .map(move |hex| {
                let hex = center + hex;

                let q = hex.x as f64 * (self.layout.scale.x as f64) / self.chunk_radius as f64;
                let r = hex.y as f64 * (self.layout.scale.y as f64) / self.chunk_radius as f64;
                let s = -(q + r);
                let height = self.perlin.get([s, q, r]);
                (hex, height as f32)
            })
            .collect();

        Chunk { center, hexes }
    }

    fn center(&self, hex: Hex) -> Hex {
        hex.to_lower_res(self.chunk_radius).to_higher_res(self.chunk_radius)
    }

    fn world_pos_to_hex(&self, position: Vec2) -> Hex {
        self.layout.world_pos_to_hex(position)
    }

    fn hex_to_world_pos(&self, hex: Hex) -> Vec2 {
        self.layout.hex_to_world_pos(hex)
    }
}

#[derive(Resource, Debug, Default, Clone)]
struct HexMapStorage {
    chunks: HashMap<Hex, Chunk>,
}

impl HexMapStorage {
    fn get_chunk(&self, hex: Hex) -> Option<&Chunk> {
        self.chunks.get(&hex)
    }

    fn insert_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.center, chunk);
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HexMapPerlinSet;

pub struct HexMapPerlinPlugin {
    pub seed: u32,
    pub hex_size: f32,
    pub chunk_radius: u32,
}

impl HexMapPerlinPlugin {
    pub fn new(seed: u32, hex_size: f32, chunk_radius: u32) -> Self {
        Self {
            seed,
            hex_size,
            chunk_radius,
        }
    }
}

impl Plugin for HexMapPerlinPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HexMapGenerator {
            layout: HexLayout {
                scale: Vec2::splat(self.hex_size),
                ..default()
            },
            perlin: Perlin::new(self.seed),
            chunk_radius: self.chunk_radius,
        });
        app.insert_resource(HexMapStorage::default());

        app.add_systems(Update, (generate_chunks).in_set(HexMapPerlinSet).chain());
    }
}

fn generate_chunks(
    mut commands: Commands,
    mut storage: ResMut<HexMapStorage>,
    generator: Res<HexMapGenerator>,
    q_probe: Query<&Transform, With<HexProbe>>,
) {
    for transform in q_probe.iter() {
        let hex = generator.world_pos_to_hex(transform.translation.xz());
        let center = generator.center(hex);

        if let Some(_) = storage.get_chunk(center) {
            continue;
        }

        let chunk = generator.chunk(center);
        storage.insert_chunk(chunk.clone());

        for (hex, height) in chunk.hexes {
            let pos = generator.hex_to_world_pos(hex).extend(0.0).xzy();

            commands.spawn((
                HexCoord(hex),
                HexNoise(height),
                Transform::from_translation(pos),
                Name::new("Hex"),
            ));
        }
    }
}
