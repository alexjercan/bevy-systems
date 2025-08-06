//! TODO: Document this module

use bevy::{platform::collections::HashMap, prelude::*};
use hexx::*;
use noise::NoiseFn;

#[cfg(feature = "debug")]
use self::debug::{DebugPlugin, DebugSet};

#[derive(Event, Clone, Debug)]
pub struct HexDiscoverEvent(pub Vec2);

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct HexCoord(pub Hex);

#[derive(Debug, Clone)]
struct Chunk {
    center: Hex,
    hexes: Vec<Hex>,
}

#[derive(Resource, Debug, Clone)]
struct HexMapStorage {
    layout: HexLayout,
    chunk_radius: u32,
    discover_radius: u32,
    chunks: HashMap<Hex, Chunk>,
}

impl HexMapStorage {
    fn discover(&self, center: Hex) -> Vec<Hex> {
        shapes::hexagon(Hex::ZERO, self.discover_radius)
            .map(|hex| hex.to_higher_res(self.chunk_radius))
            .map(|hex| center + hex)
            .collect()
    }

    fn chunk(&self, center: Hex) -> Chunk {
        let hexes = shapes::hexagon(Hex::ZERO, self.chunk_radius)
            .map(move |hex| center + hex)
            .collect();

        Chunk { center, hexes }
    }

    fn center(&self, hex: &Hex) -> Hex {
        hex.to_lower_res(self.chunk_radius)
            .to_higher_res(self.chunk_radius)
    }

    fn world_pos_to_hex(&self, position: Vec2) -> Hex {
        self.layout.world_pos_to_hex(position)
    }

    fn hex_to_world_pos(&self, hex: Hex) -> Vec2 {
        self.layout.hex_to_world_pos(hex)
    }

    fn get_chunk(&self, hex: Hex) -> Option<&Chunk> {
        self.chunks.get(&hex)
    }

    fn insert_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.center, chunk);
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HexMapSet;

pub struct HexMapPlugin {
    layout: HexLayout,
    chunk_radius: u32,
    discover_radius: u32,
}

impl HexMapPlugin {
    pub fn new(layout: HexLayout, chunk_radius: u32, discover_radius: u32) -> Self {
        Self {
            layout,
            chunk_radius,
            discover_radius,
        }
    }
}

impl Plugin for HexMapPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.add_plugins(DebugPlugin);
        #[cfg(feature = "debug")]
        app.configure_sets(Update, DebugSet.in_set(HexMapSet));

        app.add_event::<HexDiscoverEvent>();

        app.insert_resource(HexMapStorage {
            layout: self.layout.clone(),
            chunk_radius: self.chunk_radius,
            discover_radius: self.discover_radius,
            chunks: HashMap::default(),
        });

        app.add_systems(Update, (generate_chunks).in_set(HexMapSet).chain());
    }
}

fn generate_chunks(
    mut commands: Commands,
    mut storage: ResMut<HexMapStorage>,
    mut ev_discover: EventReader<HexDiscoverEvent>,
) {
    for HexDiscoverEvent(pos) in ev_discover.read() {
        let hex = storage.world_pos_to_hex(*pos);
        let center = storage.center(&hex);

        for center in storage.discover(center) {
            if let Some(_) = storage.get_chunk(center) {
                continue;
            }

            let chunk = storage.chunk(center);
            storage.insert_chunk(chunk.clone());

            for hex in chunk.hexes {
                let pos = storage.hex_to_world_pos(hex).extend(0.0).xzy();

                commands.spawn((
                    HexCoord(hex),
                    Transform::from_translation(pos),
                    Name::new("Hex"),
                ));
            }
        }
    }
}

pub trait FromNoise: Component {
    fn from_noise(value: f32) -> Self;
}

#[derive(Resource, Debug, Clone)]
struct HexMapGenerator<F: NoiseFn<f64, 3>> {
    func: F,
}

impl<F: NoiseFn<f64, 3>> HexMapGenerator<F> {
    fn noise(&self, hex: Hex) -> f32 {
        let q = hex.x as f64;
        let r = hex.y as f64;
        let s = -(q + r);

        self.func.get([q, r, s]) as f32
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HexMapNoiseSet;

pub struct HexMapNoisePlugin<F: NoiseFn<f64, 3>, C: FromNoise> {
    func: F,
    _marker: std::marker::PhantomData<C>,
}

impl<F: NoiseFn<f64, 3>, C: FromNoise> HexMapNoisePlugin<F, C> {
    pub fn new(func: F) -> Self {
        Self {
            func,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<F: NoiseFn<f64, 3> + Copy + Send + Sync + 'static, C: FromNoise + Send + Sync + 'static> Plugin
    for HexMapNoisePlugin<F, C>
{
    fn build(&self, app: &mut App) {
        app.insert_resource(HexMapGenerator { func: self.func });

        app.add_systems(
            Update,
            (generate_noise::<F, C>).in_set(HexMapNoiseSet).chain(),
        );
    }
}

fn generate_noise<
    F: NoiseFn<f64, 3> + Send + Sync + 'static,
    C: FromNoise + Send + Sync + 'static,
>(
    mut commands: Commands,
    generator: Res<HexMapGenerator<F>>,
    q_hex: Query<(Entity, &HexCoord), Without<C>>,
) {
    for (entity, coord) in q_hex.iter() {
        let noise = generator.noise(**coord);

        commands.entity(entity).insert(C::from_noise(noise));
    }
}

mod debug {
    use super::{HexCoord, HexMapStorage};
    use bevy::prelude::*;

    #[derive(Debug, Resource, Default, Clone, Deref, DerefMut)]
    struct ShowGrid(pub bool);

    #[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
    pub struct DebugSet;

    pub struct DebugPlugin;

    impl Plugin for DebugPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(ShowGrid(true));
            app.add_systems(Update, (toggle, draw_grid).in_set(DebugSet));
        }
    }

    fn toggle(kbd: Res<ButtonInput<KeyCode>>, mut show_grid: ResMut<ShowGrid>) {
        if kbd.just_pressed(KeyCode::F12) {
            show_grid.0 = !show_grid.0;
        }
    }

    fn draw_grid(
        mut gizmos: Gizmos,
        q_hex: Query<&HexCoord>,
        show_grid: Res<ShowGrid>,
        storage: Res<HexMapStorage>,
    ) {
        if !**show_grid {
            return;
        }

        for hex in q_hex.iter() {
            let pos = storage.hex_to_world_pos(**hex).extend(0.0).xzy();
            let size = storage.layout.scale.x;

            let mut direction = Vec3::new(-size, 0.0, 0.0);
            let rotation = Quat::from_rotation_y(std::f32::consts::PI / 3.0);
            for _ in 0..6 {
                let prev = pos + direction;
                direction = rotation.mul_vec3(direction);
                let next = pos + direction;
                gizmos.line(prev, next, Color::WHITE);
            }
        }
    }
}
