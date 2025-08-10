//! Hexagonal map plugin for Bevy.
//!
//! This plugin provides a hexagonal map system that allows for the discovery and management of
//! hexagonal tiles in a Bevy application. The hex tiles are organized into chunks, and the plugin
//! allows for the dynamic discovery of hexes based on a specified layout and radius.
//!
//! This plugin uses ECS style architecture to manage hexagonal tiles, where each hexagon is
//! represented by an entity with a component that can be constructed from a `Hex` coordinate.
//! The chunks are also entities that contain all the hexes within a certain radius.
//!
//! You can use the `debug` feature to enable debug visualization of the hexagonal grid.

use bevy::{platform::collections::HashMap, prelude::*};
use hexx::*;

pub mod prelude {
    pub use super::{
        HexDiscoverEvent, HexMapPlugin, HexMapSet,
    };
}

#[cfg(feature = "debug")]
use self::debug::{DebugPlugin, DebugSet};

/// The HexDiscoverEvent is used to trigger the discovery of hexagonal tiles in the map.
/// The position is given in world coordinates, and the event is generic over a component type `C`
/// that can be constructed from a `Hex` coordinate.
#[derive(Event, Clone, Debug)]
pub struct HexDiscoverEvent<C: From<Hex>> {
    /// The position in world coordinates where the discovery event occurs.
    pub pos: Vec2,
    _marker: std::marker::PhantomData<C>,
}

impl<C: From<Hex>> HexDiscoverEvent<C> {
    /// Creates a new HexDiscoverEvent with the specified position.
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            _marker: std::marker::PhantomData,
        }
    }
}

#[derive(Component, Clone, Debug, Deref, DerefMut)]
struct ChunkCoord(pub Hex);

#[derive(Resource, Debug, Clone)]
struct HexMapStorage {
    layout: HexLayout,
    chunk_radius: u32,
    discover_radius: u32,
    chunks: HashMap<Hex, Entity>,
    hexes: HashMap<Hex, Entity>,
}

impl HexMapStorage {
    fn discover_chunks(&self, center: Hex) -> Vec<Hex> {
        shapes::hexagon(Hex::ZERO, self.discover_radius)
            .map(|hex| hex.to_higher_res(self.chunk_radius))
            .map(|hex| center + hex)
            .collect()
    }

    fn chunk_hexes(&self, center: Hex) -> Vec<Hex> {
        shapes::hexagon(Hex::ZERO, self.chunk_radius)
            .map(move |hex| center + hex)
            .collect()
    }

    fn hex_to_center(&self, hex: &Hex) -> Hex {
        hex.to_lower_res(self.chunk_radius)
            .to_higher_res(self.chunk_radius)
    }

    pub fn world_pos_to_hex(&self, position: Vec2) -> Hex {
        self.layout.world_pos_to_hex(position)
    }

    pub fn hex_to_world_pos(&self, hex: Hex) -> Vec2 {
        self.layout.hex_to_world_pos(hex)
    }

    fn get_chunk(&self, hex: Hex) -> Option<&Entity> {
        self.chunks.get(&hex)
    }

    fn insert_chunk(&mut self, center: Hex, chunk: Entity) {
        self.chunks.insert(center, chunk);
    }

    fn insert_hex(&mut self, hex: Hex, entity: Entity) {
        self.hexes.insert(hex, entity);
    }
}

/// The HexMapSet is a system set used to group hex map related systems together.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HexMapSet;

/// The HexMapPlugin is a Bevy plugin that sets up the hexagonal map system.
/// The plugin requires a component type `C` that can be constructed from a `Hex` coordinate.
/// The plugin will generate a hexagonal grid based on the specified layout, chunk radius, and
/// discover radius. It will spawn a new entity for each hexagon in the discovered chunks and
/// it will add the `C` component to each hexagon entity.
/// The hexagons will be grouped into chunks, and each chunk will be represented by a `ChunkCoord`
/// component. Each tile in the chunk will be parented to the chunk entity.
pub struct HexMapPlugin<C: From<Hex>> {
    layout: HexLayout,
    chunk_radius: u32,
    discover_radius: u32,
    _marker: std::marker::PhantomData<C>,
}

impl<C: From<Hex>> HexMapPlugin<C> {
    pub fn new(layout: HexLayout, chunk_radius: u32, discover_radius: u32) -> Self {
        Self {
            layout,
            chunk_radius,
            discover_radius,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C: Component + From<Hex> + Send + Sync + 'static> Plugin for HexMapPlugin<C> {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.add_plugins(DebugPlugin);
        #[cfg(feature = "debug")]
        app.configure_sets(Update, DebugSet.in_set(HexMapSet));

        app.add_event::<HexDiscoverEvent<C>>();

        app.insert_resource(HexMapStorage {
            layout: self.layout.clone(),
            chunk_radius: self.chunk_radius,
            discover_radius: self.discover_radius,
            chunks: HashMap::default(),
            hexes: HashMap::default(),
        });

        app.add_systems(Update, (generate_chunks::<C>).in_set(HexMapSet).chain());
    }
}

fn generate_chunks<C: Component + From<Hex> + Send + Sync + 'static>(
    mut commands: Commands,
    mut storage: ResMut<HexMapStorage>,
    mut ev_discover: EventReader<HexDiscoverEvent<C>>,
) {
    for ev in ev_discover.read() {
        let hex = storage.world_pos_to_hex(ev.pos);
        let center = storage.hex_to_center(&hex);

        for center in storage.discover_chunks(center) {
            if let Some(_) = storage.get_chunk(center) {
                continue;
            }

            let pos = storage.hex_to_world_pos(center).extend(0.0).xzy();
            let chunk_entity = commands
                .spawn((
                    ChunkCoord(center),
                    Transform::from_translation(pos),
                    Visibility::default(),
                    Name::new("HexChunk"),
                ))
                .id();
            storage.insert_chunk(center, chunk_entity);

            for hex in storage.chunk_hexes(center) {
                let pos = storage.hex_to_world_pos(hex - center).extend(0.0).xzy();

                let hex_entity = commands
                    .spawn((
                        C::from(hex),
                        Transform::from_translation(pos),
                        Name::new("Hex"),
                    ))
                    .id();
                commands.entity(chunk_entity).add_child(hex_entity);
                storage.insert_hex(hex, hex_entity);
            }
        }
    }
}

mod debug {
    use super::{HexMapStorage, ChunkCoord};
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
        q_chunk: Query<&ChunkCoord>,
        show_grid: Res<ShowGrid>,
        storage: Res<HexMapStorage>,
    ) {
        if !**show_grid {
            return;
        }

        for chunk in q_chunk.iter() {
            for hex in storage.chunk_hexes(**chunk) {
                let pos = storage.hex_to_world_pos(hex).extend(0.0).xzy();
                draw_hex(&mut gizmos, pos, storage.layout.scale.x, Color::WHITE);
            }

            let pos = storage.hex_to_world_pos(**chunk).extend(0.0).xzy();
            draw_hex(&mut gizmos, pos, storage.layout.scale.x * 0.5, Color::srgb_u8(255, 255, 0));
        }
    }

    fn draw_hex(gizmos: &mut Gizmos, pos: Vec3, size: f32, color: Color) {
        let mut direction = Vec3::new(-size, 0.0, 0.0);
        let rotation = Quat::from_rotation_y(std::f32::consts::PI / 3.0);
        for _ in 0..6 {
            let prev = pos + direction;
            direction = rotation.mul_vec3(direction);
            let next = pos + direction;
            gizmos.line(prev, next, color);
        }
    }
}
