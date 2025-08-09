//! TODO: Hexmap coordinates docs with simple mesh

#[path = "../helpers/wasd_camera_controller.rs"]
mod wasd_camera_controller;

#[path = "common.rs"]
mod common;

use bevy::{
    asset::RenderAssetUsages,
    platform::collections::HashMap,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use hexx::*;

use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use systems::{debug::DebugPlugin, hexmap::prelude::*, noise::prelude::*};

use wasd_camera_controller::{WASDCameraControllerBundle, WASDCameraControllerPlugin};
use common::HexCoord;

const HEX_SIZE: f32 = 1.0;
const CHUNK_RADIUS: u32 = 15;
const DISCOVER_RADIUS: u32 = 3;

#[derive(Component, Debug, Clone, Copy)]
struct HexNoise(f64);

impl From<f64> for HexNoise {
    fn from(noise: f64) -> Self {
        Self(noise)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
enum TileKind {
    Mountains,
    Hills,
    Plains,
    Sand,
    Water,
    DeepWater,
}

impl Into<Color> for TileKind {
    fn into(self) -> Color {
        match self {
            TileKind::Mountains => Color::srgb_u8(255, 255, 255),
            TileKind::Hills => Color::srgb_u8(139, 69, 19),
            TileKind::Plains => Color::srgb_u8(0, 128, 0),
            TileKind::Sand => Color::srgb_u8(255, 255, 0),
            TileKind::Water => Color::srgb_u8(0, 0, 255),
            TileKind::DeepWater => Color::srgb_u8(0, 0, 139),
        }
    }
}

impl From<f64> for TileKind {
    fn from(value: f64) -> Self {
        if value <= -0.5 {
            TileKind::DeepWater
        } else if value <= 0.0 {
            TileKind::Water
        } else if value <= 0.1 {
            TileKind::Sand
        } else if value <= 0.3 {
            TileKind::Plains
        } else if value <= 0.6 {
            TileKind::Hills
        } else {
            TileKind::Mountains
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
struct AssetsCache {
    mesh: Handle<Mesh>,
    materials: HashMap<TileKind, Handle<StandardMaterial>>,
    layout: HexLayout,
}

impl AssetsCache {
    fn hexagonal_column(&self) -> Mesh {
        const COLUMN_HEIGHT: f32 = 5.0;

        let mesh_info = ColumnMeshBuilder::new(&self.layout, COLUMN_HEIGHT)
            .without_bottom_face()
            .center_aligned()
            .build();
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs)
        .with_inserted_indices(Indices::U16(mesh_info.indices))
    }
}

fn main() {
    let layout = HexLayout::flat().with_hex_size(HEX_SIZE);

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HexMapPlugin::<HexCoord>::new(
            layout.clone(),
            CHUNK_RADIUS,
            DISCOVER_RADIUS,
        ))
        .add_plugins(NoisePlugin::<3, HexCoord, _, HexNoise>::new(
            Planet::default().with_seed(CURRENT_SEED),
        ))
        .add_plugins(WASDCameraControllerPlugin)
        .add_plugins(DebugPlugin)
        .insert_resource(AssetsCache {
            layout,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (input, handle_hex))
        .configure_sets(Update, HexMapSet)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut assets_cache: ResMut<AssetsCache>,
) {
    commands.spawn((
        WASDCameraControllerBundle::default(),
        Camera3d::default(),
        Transform::from_xyz(60.0, 60.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("RTS Camera"),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(60.0, 60.0, 00.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Directional Light"),
    ));

    assets_cache.mesh = meshes.add(assets_cache.hexagonal_column());
    for tile in [
        TileKind::Mountains,
        TileKind::Hills,
        TileKind::Plains,
        TileKind::Sand,
        TileKind::Water,
        TileKind::DeepWater,
    ] {
        assets_cache.materials.insert(
            tile,
            materials.add(StandardMaterial {
                base_color: tile.into(),
                perceptual_roughness: 1.0,
                metallic: 0.0,
                ..default()
            }),
        );
    }
}

fn input(
    windows: Query<&Window>,
    q_camera: Single<(&Camera, &GlobalTransform)>,
    mut ev_discover: EventWriter<HexDiscoverEvent<HexCoord>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let (camera, transform) = q_camera.into_inner();

    let Some(cursor_position) = windows.single().unwrap().cursor_position() else {
        return;
    };

    let Ok(ray) = camera.viewport_to_world(transform, cursor_position) else {
        return;
    };

    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else {
        return;
    };
    let point = ray.get_point(distance);

    ev_discover.write(HexDiscoverEvent::new(point.xz()));
}

fn handle_hex(
    mut commands: Commands,
    mut q_hex: Query<(Entity, &HexNoise, &mut Transform), (With<HexCoord>, Without<Mesh3d>)>,
    assets_cache: Res<AssetsCache>,
) {
    for (entity, HexNoise(height), mut transform) in q_hex.iter_mut() {
        let tile = TileKind::from(*height);
        let height_value = height.clamp(0.0, 1.0);
        let height_value = height_value as f32 * 2.0;

        commands.entity(entity).insert((
            Mesh3d(assets_cache.mesh.clone()),
            MeshMaterial3d(
                assets_cache
                    .materials
                    .get(&tile)
                    .cloned()
                    .unwrap_or_default(),
            ),
        ));

        transform.translation.y = height_value;
    }
}

/// Planet seed. Change this to generate a different planet.
const CURRENT_SEED: u32 = 0;

/// Scale of the planet. Change this to zoom in or out.
const ZOOM_SCALE: f64 = 0.01;

/// Frequency of the planet's continents. Higher frequency produces
/// smaller, more numerous continents. This value is measured in radians.
const CONTINENT_FREQUENCY: f64 = 1.0;

/// Lacunarity of the planet's continents. Changing this value produces
/// slightly different continents. For the best results, this value should
/// be random, but close to 2.0.
const CONTINENT_LACUNARITY: f64 = 2.208984375;

/// Lacunarity of the planet's mountains. Changing the value produces
/// slightly different mountains. For the best results, this value should
/// be random, but close to 2.0.
const MOUNTAIN_LACUNARITY: f64 = 2.142578125;

/// Lacunarity of the planet's hills. Changing this value produces
/// slightly different hills. For the best results, this value should be
/// random, but close to 2.0.
const HILLS_LACUNARITY: f64 = 2.162109375;

/// Lacunarity of the planet's plains. Changing this value produces
/// slightly different plains. For the best results, this value should be
/// random, but close to 2.0.
const PLAINS_LACUNARITY: f64 = 2.314453125;

/// Lacunarity of the planet's badlands. Changing this value produces
/// slightly different badlands. For the best results, this value should
/// be random, but close to 2.0.
const BADLANDS_LACUNARITY: f64 = 2.212890625;

/// Specifies the "twistiness" of the mountains.
const MOUNTAINS_TWIST: f64 = 1.0;

/// Specifies the "twistiness" of the hills.
const HILLS_TWIST: f64 = 1.0;

/// Specifies the "twistiness" of the badlands.
const BADLANDS_TWIST: f64 = 1.0;

/// Specifies the planet's sea level. This value must be between -1.0
/// (minimum planet elevation) and +1.0 (maximum planet elevation).
const SEA_LEVEL: f64 = 0.0;

/// Specifies the level on the planet in which continental shelves appear.
/// This value must be between -1.0 (minimum planet elevation) and +1.0
/// (maximum planet elevation), and must be less than `SEA_LEVEL`.
const SHELF_LEVEL: f64 = -0.375;

/// Determines the amount of mountainous terrain that appears on the
/// planet. Values range from 0.0 (no mountains) to 1.0 (all terrain is
/// covered in mountains). Mountains terrain will overlap hilly terrain.
/// Because the badlands terrain may overlap parts of the mountainous
/// terrain, setting `MOUNTAINS_AMOUNT` to 1.0 may not completely cover the
/// terrain in mountains.
const MOUNTAINS_AMOUNT: f64 = 0.5;

/// Determines the amount of hilly terrain that appears on the planet.
/// Values range from 0.0 (no hills) to 1.0 (all terrain is covered in
/// hills). This value must be less than `MOUNTAINS_AMOUNT`. Because the
/// mountains terrain will overlap parts of the hilly terrain, and the
/// badlands terrain may overlap parts of the hilly terrain, setting
/// `HILLS_AMOUNT` to 1.0 may not completely cover the terrain in hills.
const HILLS_AMOUNT: f64 = (1.0 + MOUNTAINS_AMOUNT) / 2.0;

/// Determines the amount of badlands terrain that covers the planet.
/// Values range from 0.0 (no badlands) to 1.0 (all terrain is covered in
/// badlands). Badlands terrain will overlap any other type of terrain.
const BADLANDS_AMOUNT: f64 = 0.3125;

/// Offset to apply to the terrain type definition. Low values (< 1.0)
/// cause the rough areas to appear only at high elevations. High values
/// (> 2.0) cause the rough areas to appear at any elevation. The
/// percentage of rough areas on the planet are independent of this value.
const TERRAIN_OFFSET: f64 = 1.0;

/// Specifies the amount of "glaciation" on the mountains. This value
/// should be close to 1.0 and greater than 1.0.
const MOUNTAIN_GLACIATION: f64 = 1.375;

/// Scaling to apply to the base continent elevations, in planetary
/// elevation units.
const CONTINENT_HEIGHT_SCALE: f64 = (1.0 - SEA_LEVEL) / 4.0;

/// Maximum depth of the rivers, in planetary elevation units.
const RIVER_DEPTH: f64 = 0.0234375;

#[derive(Clone, Copy, Debug)]
struct Planet {
    seed: u32,
    zoom_scale: f64,
    continent_frequency: f64,
    continent_lacunarity: f64,
    mountain_lacunarity: f64,
    hills_lacunarity: f64,
    plains_lacunarity: f64,
    badlands_lacunarity: f64,
    mountains_twist: f64,
    hills_twist: f64,
    badlands_twist: f64,
    sea_level: f64,
    shelf_level: f64,
    mountains_amount: f64,
    hills_amount: f64,
    badlands_amount: f64,
    terrain_offset: f64,
    mountain_glaciation: f64,
    continent_height_scale: f64,
    river_depth: f64,
}

impl Default for Planet {
    fn default() -> Self {
        Planet {
            seed: CURRENT_SEED,
            zoom_scale: ZOOM_SCALE,
            continent_frequency: CONTINENT_FREQUENCY,
            continent_lacunarity: CONTINENT_LACUNARITY,
            mountain_lacunarity: MOUNTAIN_LACUNARITY,
            hills_lacunarity: HILLS_LACUNARITY,
            plains_lacunarity: PLAINS_LACUNARITY,
            badlands_lacunarity: BADLANDS_LACUNARITY,
            mountains_twist: MOUNTAINS_TWIST,
            hills_twist: HILLS_TWIST,
            badlands_twist: BADLANDS_TWIST,
            sea_level: SEA_LEVEL,
            shelf_level: SHELF_LEVEL,
            mountains_amount: MOUNTAINS_AMOUNT,
            hills_amount: HILLS_AMOUNT,
            badlands_amount: BADLANDS_AMOUNT,
            terrain_offset: TERRAIN_OFFSET,
            mountain_glaciation: MOUNTAIN_GLACIATION,
            continent_height_scale: CONTINENT_HEIGHT_SCALE,
            river_depth: RIVER_DEPTH,
        }
    }
}

impl Planet {
    fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }
}

impl NoiseFn<f64, 3> for Planet {
    fn get(&self, point: [f64; 3]) -> f64 {
        _ = self.mountain_lacunarity; // Silence unused warning
        _ = self.hills_lacunarity; // Silence unused warning
        _ = self.plains_lacunarity; // Silence unused warning
        _ = self.badlands_lacunarity; // Silence unused warning
        _ = self.mountains_twist; // Silence unused warning
        _ = self.hills_twist; // Silence unused warning
        _ = self.badlands_twist; // Silence unused warning
        _ = self.shelf_level; // Silence unused warning
        _ = self.mountain_glaciation; // Silence unused warning
        _ = self.river_depth; // Silence unused warning
        _ = self.terrain_offset; // Silence unused warning
        _ = self.hills_amount; // Silence unused warning
        _ = self.mountains_amount; // Silence unused warning
        _ = self.badlands_amount; // Silence unused warning
        _ = self.continent_height_scale; // Silence unused warning

        // Example taken from
        // <https://github.com/Razaekel/noise-rs/blob/develop/examples/complexplanet.rs>

        // 1: [Continent module]: This FBM module generates the continents. This
        // noise function has a high number of octaves so that detail is visible at
        // high zoom levels.
        let base_continent_def_fb0 = Fbm::<Perlin>::new(self.seed)
            .set_frequency(self.continent_frequency)
            .set_persistence(0.5)
            .set_lacunarity(self.continent_lacunarity)
            .set_octaves(14);

        // 2: [Continent-with-ranges module]: Next, a curve module modifies the
        // output value from the continent module so that very high values appear
        // near sea level. This defines the positions of the mountain ranges.
        let base_continent_def_cu = noise::Curve::new(base_continent_def_fb0)
            .add_control_point(-2.0000 + self.sea_level, -1.625 + self.sea_level)
            .add_control_point(-1.0000 + self.sea_level, -1.375 + self.sea_level)
            .add_control_point(0.0000 + self.sea_level, -0.375 + self.sea_level)
            .add_control_point(0.0625 + self.sea_level, 0.125 + self.sea_level)
            .add_control_point(0.1250 + self.sea_level, 0.250 + self.sea_level)
            .add_control_point(0.2500 + self.sea_level, 1.000 + self.sea_level)
            .add_control_point(0.5000 + self.sea_level, 0.250 + self.sea_level)
            .add_control_point(0.7500 + self.sea_level, 0.250 + self.sea_level)
            .add_control_point(1.0000 + self.sea_level, 0.500 + self.sea_level)
            .add_control_point(2.0000 + self.sea_level, 0.500 + self.sea_level);

        // 3: [Carver module]: This higher-frequency BasicMulti module will be
        // used by subsequent noise functions to carve out chunks from the
        // mountain ranges within the continent-with-ranges module so that the
        // mountain ranges will not be completely impassible.
        let base_continent_def_fb1 = Fbm::<Perlin>::new(self.seed + 1)
            .set_frequency(self.continent_frequency * 4.34375)
            .set_persistence(0.5)
            .set_lacunarity(self.continent_lacunarity)
            .set_octaves(11);

        // 4: [Scaled-carver module]: This scale/bias module scales the output
        // value from the carver module such that it is usually near 1.0. This
        // is required for step 5.
        let base_continent_def_sb = noise::ScaleBias::new(base_continent_def_fb1)
            .set_scale(0.375)
            .set_bias(0.625);

        // 5: [Carved-continent module]: This minimum-value module carves out
        // chunks from the continent-with-ranges module. it does this by ensuring
        // that only the minimum of the output values from the scaled-carver
        // module and the continent-with-ranges module contributes to the output
        // value of this subgroup. Most of the time, the minimum value module will
        // select the output value from the continent-with-ranges module since the
        // output value from the scaled-carver is usually near 1.0. Occasionally,
        // the output from the scaled-carver module will be less than the output
        // value from the continent-with-ranges module, so in this case, the output
        // value from the scaled-carver module is selected.
        let base_continent_def_mi = noise::Min::new(base_continent_def_sb, base_continent_def_cu);

        // 6: [Clamped-continent module]: Finally, a clamp module modifies the
        // carved continent module to ensure that the output value of this subgroup
        // is between -1.0 and 1.0.
        let base_continent_def_cl = noise::Clamp::new(base_continent_def_mi).set_bounds(-1.0, 1.0);

        // 7: [Base-continent-definition subgroup]: Caches the output value from
        // the clamped-continent module.
        let base_continent_def = noise::Cache::new(base_continent_def_cl);

        let x = point[0] * self.zoom_scale;
        let y = point[1] * self.zoom_scale;
        let z = point[2] * self.zoom_scale;

        base_continent_def.get([x, y, z])
    }
}
