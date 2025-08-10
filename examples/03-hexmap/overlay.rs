//! TODO: Hexmap coordinates docs with simple mesh

#[path = "../helpers/wasd_camera_controller.rs"]
mod wasd_camera_controller;

#[path = "common.rs"]
mod common;

use bevy::{
    asset::RenderAssetUsages,
    pbr::{ExtendedMaterial, MaterialExtension},
    platform::collections::HashMap,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_resource::{AsBindGroup, ShaderRef},
        storage::ShaderStorageBuffer,
    },
};
use hexx::*;

use itertools::Itertools;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use systems::{debug::DebugPlugin, hexmap::prelude::*, noise::prelude::*};

use common::HexCoord;
use wasd_camera_controller::{WASDCameraControllerBundle, WASDCameraControllerPlugin};

const HEX_SIZE: f32 = 1.0;
const CHUNK_RADIUS: u32 = 15;
const DISCOVER_RADIUS: u32 = 3;
const COLUMN_HEIGHT: f32 = 5.0;

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
struct HexNoiseHeight(f64);

impl From<f64> for HexNoiseHeight {
    fn from(noise: f64) -> Self {
        Self(noise)
    }
}

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
struct HexNoiseTemperature(f64);

impl From<f64> for HexNoiseTemperature {
    fn from(noise: f64) -> Self {
        Self(noise)
    }
}

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
struct HexNoiseHumidity(f64);

impl From<f64> for HexNoiseHumidity {
    fn from(noise: f64) -> Self {
        Self(noise)
    }
}

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
enum BiomeKind {
    DeepOcean,
    Ocean,
    Desert,
    Grassland,
    SnowyPlains,
    Barren,
    Forest,
    SnowyHills,
    SnowyForest,
    Mountains,
    SnowyMountains,
}

impl BiomeKind {
    fn from_noise(elevation: f64, temperature: f64, humidity: f64) -> Self {
        match (elevation, temperature, humidity) {
            (e, _, _) if e < -0.5 => BiomeKind::DeepOcean,
            (e, _, _) if e < 0.0 => BiomeKind::Ocean,
            (e, t, _) if e < 0.3 && t >= 0.7 => BiomeKind::Desert,
            (e, t, _) if e < 0.3 && t >= 0.2 && t < 0.7 => BiomeKind::Grassland,
            (e, t, _) if e < 0.3 && t < 0.2 => BiomeKind::SnowyPlains,
            (e, t, h) if e < 0.8 && t >= 0.3 && h < 0.5 => BiomeKind::Barren,
            (e, t, h) if e < 0.8 && t >= 0.3 && h >= 0.5 => BiomeKind::Forest,
            (e, t, h) if e < 0.8 && t < 0.3 && h < 0.5 => BiomeKind::SnowyHills,
            (e, t, h) if e < 0.8 && t < 0.3 && h >= 0.5 => BiomeKind::SnowyForest,
            (e, t, _) if e >= 0.8 && t >= 0.5 => BiomeKind::Mountains,
            (e, t, _) if e >= 0.8 && t < 0.5 => BiomeKind::SnowyMountains,
            _ => unreachable!(),
        }
    }

    fn to_color(self) -> LinearRgba {
        match self {
            BiomeKind::DeepOcean => LinearRgba::new(0.0, 0.18, 0.35, 1.0), // very dark blue-green
            BiomeKind::Ocean => LinearRgba::new(0.0, 0.3, 0.5, 1.0),       // deep blue-green ocean
            BiomeKind::Desert => LinearRgba::new(0.85, 0.73, 0.5, 1.0),    // warm sandy tan
            BiomeKind::Grassland => LinearRgba::new(0.4, 0.65, 0.3, 1.0),  // muted grassy green
            BiomeKind::SnowyPlains => LinearRgba::new(0.95, 0.95, 0.96, 1.0), // bright snow with a hint of blue
            BiomeKind::Barren => LinearRgba::new(0.45, 0.4, 0.35, 1.0),       // rocky brown-gray
            BiomeKind::Forest => LinearRgba::new(0.15, 0.35, 0.15, 1.0),      // dark evergreen
            BiomeKind::SnowyHills => LinearRgba::new(0.9, 0.92, 0.94, 1.0), // snowy but slightly shaded
            BiomeKind::SnowyForest => LinearRgba::new(0.75, 0.8, 0.78, 1.0), // snow-dusted trees
            BiomeKind::Mountains => LinearRgba::new(0.45, 0.45, 0.45, 1.0), // rocky gray
            BiomeKind::SnowyMountains => LinearRgba::new(0.92, 0.92, 0.94, 1.0), // snow-capped peaks
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect, Default)]
enum OverlayKind {
    Height,
    Temperature,
    Humidity,
    #[default]
    Tile,
}

#[derive(Resource, Debug, Clone, Default)]
struct OverlayState {
    kind: OverlayKind,
}

fn main() {
    let layout = HexLayout::flat().with_hex_size(HEX_SIZE);
    let seed = CURRENT_SEED;

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HexMapPlugin::<HexCoord>::new(
            layout.clone(),
            CHUNK_RADIUS,
            DISCOVER_RADIUS,
        ))
        .add_plugins(NoisePlugin::<3, HexCoord, _, HexNoiseHeight>::new(
            PlanetHeight::default().with_seed(seed),
        ))
        .add_plugins(NoisePlugin::<3, HexCoord, _, HexNoiseTemperature>::new(
            PlanetTemperature::default().with_seed(seed + 1),
        ))
        .add_plugins(NoisePlugin::<3, HexCoord, _, HexNoiseHumidity>::new(
            PlanetHumidity::default().with_seed(seed + 2),
        ))
        .add_plugins(OverlayMapMeshPlugin::new(
            layout.clone(),
            CHUNK_RADIUS,
            COLUMN_HEIGHT,
        ))
        .add_plugins(WASDCameraControllerPlugin)
        .add_plugins(DebugPlugin)
        .configure_sets(Update, HexMapSet)
        .configure_sets(Update, NoiseSet)
        .insert_resource(OverlayState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (mouse_click_discover, input_switch_overlay))
        .run();
}

fn setup(mut commands: Commands) {
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
}

fn mouse_click_discover(
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

fn input_switch_overlay(keys: Res<ButtonInput<KeyCode>>, mut overlay_state: ResMut<OverlayState>) {
    if keys.just_pressed(KeyCode::ArrowUp) {
        overlay_state.kind = match overlay_state.kind {
            OverlayKind::Height => OverlayKind::Temperature,
            OverlayKind::Temperature => OverlayKind::Humidity,
            OverlayKind::Humidity => OverlayKind::Tile,
            OverlayKind::Tile => OverlayKind::Height,
        };
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        overlay_state.kind = match overlay_state.kind {
            OverlayKind::Height => OverlayKind::Tile,
            OverlayKind::Temperature => OverlayKind::Height,
            OverlayKind::Humidity => OverlayKind::Temperature,
            OverlayKind::Tile => OverlayKind::Humidity,
        };
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
struct PlanetHeight {
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

impl Default for PlanetHeight {
    fn default() -> Self {
        PlanetHeight {
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

impl PlanetHeight {
    fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }
}

impl NoiseFn<f64, 3> for PlanetHeight {
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

#[derive(Clone, Copy, Debug)]
struct PlanetTemperature {
    seed: u32,
    zoom_scale: f64,
    continent_frequency: f64,
    continent_lacunarity: f64,
}

impl PlanetTemperature {
    fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }
}

impl Default for PlanetTemperature {
    fn default() -> Self {
        PlanetTemperature {
            seed: CURRENT_SEED,
            zoom_scale: ZOOM_SCALE,
            continent_frequency: CONTINENT_FREQUENCY,
            continent_lacunarity: CONTINENT_LACUNARITY,
        }
    }
}

impl NoiseFn<f64, 3> for PlanetTemperature {
    fn get(&self, point: [f64; 3]) -> f64 {
        let base_temperature_fb = Fbm::<Perlin>::new(self.seed)
            .set_frequency(self.continent_frequency * 0.5)
            .set_persistence(0.5)
            .set_lacunarity(self.continent_lacunarity)
            .set_octaves(8);

        let x = point[0] * self.zoom_scale;
        let y = point[1] * self.zoom_scale;
        let z = point[2] * self.zoom_scale;

        base_temperature_fb.get([x, y, z])
    }
}

#[derive(Clone, Copy, Debug)]
struct PlanetHumidity {
    seed: u32,
    zoom_scale: f64,
    continent_frequency: f64,
    continent_lacunarity: f64,
}

impl PlanetHumidity {
    fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }
}

impl Default for PlanetHumidity {
    fn default() -> Self {
        PlanetHumidity {
            seed: CURRENT_SEED,
            zoom_scale: ZOOM_SCALE,
            continent_frequency: CONTINENT_FREQUENCY,
            continent_lacunarity: CONTINENT_LACUNARITY,
        }
    }
}

impl NoiseFn<f64, 3> for PlanetHumidity {
    fn get(&self, point: [f64; 3]) -> f64 {
        let base_humidity_fb = Fbm::<Perlin>::new(self.seed)
            .set_frequency(self.continent_frequency * 0.5)
            .set_persistence(0.5)
            .set_lacunarity(self.continent_lacunarity)
            .set_octaves(8);

        let x = point[0] * self.zoom_scale;
        let y = point[1] * self.zoom_scale;
        let z = point[2] * self.zoom_scale;

        base_humidity_fb.get([x, y, z])
    }
}

#[derive(Resource, Debug, Clone, Default)]
struct HeightMapLayout {
    layout: HexLayout,
    chunk_radius: u32,
    max_height: f32,
}

impl HeightMapLayout {
    fn new(layout: HexLayout, chunk_radius: u32, max_height: f32) -> Self {
        Self {
            layout,
            chunk_radius,
            max_height,
        }
    }

    fn hexmap(&self, chunk: HashMap<Hex, f32>) -> Mesh {
        let mesh_info = HeightMapMeshBuilder::new(&self.layout, &chunk)
            .with_height_range(0.0..=self.max_height)
            .with_default_height(0.0)
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

#[derive(Component)]
struct ChunkMeshReady;

fn handle_overlay_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ChunkMaterial>>>,
    mut gradient_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, GradientMaterial>>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    layout: Res<HeightMapLayout>,
    q_hex: Query<
        (
            Entity,
            &HexCoord,
            &HexNoiseHeight,
            &HexNoiseTemperature,
            &HexNoiseHumidity,
            &ChildOf,
        ),
        Without<ChunkMeshReady>,
    >,
    overlay_state: Res<OverlayState>,
) {
    let size = layout.chunk_radius * 2 + 1;
    for (&chunk_entity, chunk) in q_hex
        .iter()
        .chunk_by(|(_, _, _, _, _, ChildOf(e))| e)
        .into_iter()
    {
        let mut center: Option<Hex> = None;
        let mut storage = HashMap::default();
        let mut color_data = vec![LinearRgba::NONE; (size * size) as usize];
        let mut height_data = vec![0.0; (size * size) as usize];
        let mut temperature_data = vec![0.0; (size * size) as usize];
        let mut humidity_data = vec![0.0; (size * size) as usize];

        for (entity, hex, noise, temperature, humidity, _) in chunk {
            commands.entity(entity).insert(ChunkMeshReady);
            let hex: Hex = hex.into();
            if center.is_none() {
                center = Some(
                    hex.to_lower_res(layout.chunk_radius)
                        .to_higher_res(layout.chunk_radius),
                );
            }
            let hex = hex - center.unwrap();

            let height = **noise;
            let temperature = **temperature;
            let humidity = **humidity;

            let height_value = (**noise).clamp(0.0, 1.0);
            let temperature_value = ((temperature + 1.0) / 2.0).clamp(0.0, 1.0);
            let temperature_value = (temperature_value - height_value * 0.2).clamp(0.0, 1.0);
            let humidity_value = ((humidity + 1.0) / 2.0).clamp(0.0, 1.0);
            let humidity_value = (humidity_value - height_value * 0.2).clamp(0.0, 1.0);

            let biome = BiomeKind::from_noise(height, temperature_value, humidity_value);

            storage.insert(hex, height_value as f32 * layout.max_height);

            let q_offset = hex.x + layout.chunk_radius as i32;
            let r_offset = hex.y + layout.chunk_radius as i32;
            let index = (r_offset * size as i32 + q_offset) as usize;
            color_data[index] = biome.to_color();
            height_data[index] = height_value as f32;
            temperature_data[index] = temperature_value as f32;
            humidity_data[index] = humidity_value as f32;
        }

        if let Some(center) = center {
            let mesh = layout.hexmap(storage);

            commands.entity(chunk_entity).with_children(|parent| {
                parent.spawn((
                    if overlay_state.kind == OverlayKind::Tile {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                    OverlayKind::Tile,
                    Mesh3d(meshes.add(mesh.clone())),
                    MeshMaterial3d(chunk_materials.add(ExtendedMaterial {
                        base: StandardMaterial {
                            perceptual_roughness: 1.0,
                            metallic: 0.0,
                            ..default()
                        },
                        extension: ChunkMaterial {
                            chunk_radius: layout.chunk_radius,
                            hex_size: layout.layout.scale.x,
                            chunk_center: IVec2::new(center.x, center.y),
                            colors: buffers.add(ShaderStorageBuffer::from(color_data)),
                        },
                    })),
                    Name::new("Overlay Height Mesh"),
                ));

                parent.spawn((
                    if overlay_state.kind == OverlayKind::Height {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                    OverlayKind::Height,
                    Mesh3d(meshes.add(mesh.clone())),
                    MeshMaterial3d(gradient_materials.add(ExtendedMaterial {
                        base: StandardMaterial {
                            perceptual_roughness: 1.0,
                            metallic: 0.0,
                            ..default()
                        },
                        extension: GradientMaterial {
                            chunk_radius: layout.chunk_radius,
                            hex_size: layout.layout.scale.x,
                            chunk_center: IVec2::new(center.x, center.y),
                            start_color: LinearRgba::BLACK,
                            end_color: LinearRgba::WHITE,
                            values: buffers.add(ShaderStorageBuffer::from(height_data)),
                        },
                    })),
                    Name::new("Overlay Height Gradient Mesh"),
                ));

                parent.spawn((
                    if overlay_state.kind == OverlayKind::Temperature {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                    OverlayKind::Temperature,
                    Mesh3d(meshes.add(mesh.clone())),
                    MeshMaterial3d(gradient_materials.add(ExtendedMaterial {
                        base: StandardMaterial {
                            perceptual_roughness: 1.0,
                            metallic: 0.0,
                            ..default()
                        },
                        extension: GradientMaterial {
                            chunk_radius: layout.chunk_radius,
                            hex_size: layout.layout.scale.x,
                            chunk_center: IVec2::new(center.x, center.y),
                            start_color: LinearRgba::BLUE,
                            end_color: LinearRgba::RED,
                            values: buffers.add(ShaderStorageBuffer::from(temperature_data)),
                        },
                    })),
                    Name::new("Overlay Temperature Gradient Mesh"),
                ));

                parent.spawn((
                    if overlay_state.kind == OverlayKind::Humidity {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                    OverlayKind::Humidity,
                    Mesh3d(meshes.add(mesh.clone())),
                    MeshMaterial3d(gradient_materials.add(ExtendedMaterial {
                        base: StandardMaterial {
                            perceptual_roughness: 1.0,
                            metallic: 0.0,
                            ..default()
                        },
                        extension: GradientMaterial {
                            chunk_radius: layout.chunk_radius,
                            hex_size: layout.layout.scale.x,
                            chunk_center: IVec2::new(center.x, center.y),
                            start_color: LinearRgba::new(1.0, 1.0, 0.0, 1.0), // Yellow
                            end_color: LinearRgba::GREEN,
                            values: buffers.add(ShaderStorageBuffer::from(humidity_data)),
                        },
                    })),
                    Name::new("Overlay Humidity Gradient Mesh"),
                ));
            });
        }
    }
}

fn handle_overlay_update(
    mut q_visible: Query<(&mut Visibility, &OverlayKind)>,
    overlay_state: Res<OverlayState>,
) {
    for (mut visibility, kind) in q_visible.iter_mut() {
        *visibility = if *kind == overlay_state.kind {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct OverlayMapMeshSet;

struct OverlayMapMeshPlugin {
    layout: HexLayout,
    chunk_radius: u32,
    max_height: f32,
}

impl OverlayMapMeshPlugin {
    pub fn new(layout: HexLayout, chunk_radius: u32, max_height: f32) -> Self {
        Self {
            layout,
            chunk_radius,
            max_height,
        }
    }
}

impl Plugin for OverlayMapMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, ChunkMaterial>,
        >::default());
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, GradientMaterial>,
        >::default());

        app.insert_resource(HeightMapLayout::new(
            self.layout.clone(),
            self.chunk_radius,
            self.max_height,
        ));

        app.add_systems(
            Update,
            (handle_overlay_chunk, handle_overlay_update).in_set(OverlayMapMeshSet),
        );
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {
    #[uniform(100)]
    pub chunk_radius: u32,
    #[uniform(101)]
    pub hex_size: f32,
    #[uniform(102)]
    pub chunk_center: IVec2,
    #[storage(103, read_only)]
    pub colors: Handle<ShaderStorageBuffer>,
}

impl MaterialExtension for ChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GradientMaterial {
    #[uniform(100)]
    pub chunk_radius: u32,
    #[uniform(101)]
    pub hex_size: f32,
    #[uniform(102)]
    pub chunk_center: IVec2,
    #[uniform(103)]
    pub start_color: LinearRgba,
    #[uniform(104)]
    pub end_color: LinearRgba,
    #[storage(105, read_only)]
    pub values: Handle<ShaderStorageBuffer>,
}

impl MaterialExtension for GradientMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/chunk_gradient.wgsl".into()
    }
}
