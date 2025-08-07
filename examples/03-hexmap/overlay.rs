//! TODO: Hexmap coordinates docs with simple mesh

#[path = "../helpers/wasd_camera_controller.rs"]
mod wasd_camera_controller;

use bevy::{
    asset::RenderAssetUsages,
    platform::collections::HashMap,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_resource::{AsBindGroup, ShaderRef},
    },
};
use hexx::*;

use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use systems::{
    debug::DebugPlugin,
    hexmap::map::{
        FromNoise, HexCoord, HexDiscoverEvent, HexMapNoisePlugin, HexMapPlugin, HexMapSet,
    },
};

use wasd_camera_controller::{WASDCameraControllerBundle, WASDCameraControllerPlugin};

#[derive(Component, Debug, Clone, Copy)]
struct RenderedHex;

const HEX_SIZE: f32 = 1.0;
const CHUNK_RADIUS: u32 = 4;
const DISCOVER_RADIUS: u32 = 3;

#[derive(Component, Debug, Clone, Copy)]
struct HexNoiseHeight(f32);

impl FromNoise for HexNoiseHeight {
    fn from_noise(noise: f32) -> Self {
        Self(noise)
    }
}

#[derive(Component, Debug, Clone, Copy)]
struct HexNoiseTemperature(f32);

impl FromNoise for HexNoiseTemperature {
    fn from_noise(noise: f32) -> Self {
        Self(noise)
    }
}

#[derive(Component, Debug, Clone, Copy)]
struct HexNoiseHumidity(f32);

impl FromNoise for HexNoiseHumidity {
    fn from_noise(noise: f32) -> Self {
        Self(noise)
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct HexMaterial {
    #[uniform(0)]
    pub mode: u32, // 0 = height, 1 = temp, 2 = humidity
    #[uniform(1)]
    pub height: f32,
    #[uniform(2)]
    pub temperature: f32,
    #[uniform(3)]
    pub humidity: f32,
    alpha_mode: AlphaMode,
}

impl Material for HexMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/hex_visualize.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

#[derive(Component, Debug, Clone, Copy)]
struct TileMesh;

#[derive(Component, Debug, Clone, Copy)]
struct OverlayMesh;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect, Default)]
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

impl From<f32> for TileKind {
    fn from(value: f32) -> Self {
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
    let seed = CURRENT_SEED;

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MaterialPlugin::<HexMaterial>::default())
        .add_plugins(HexMapPlugin::new(
            layout.clone(),
            CHUNK_RADIUS,
            DISCOVER_RADIUS,
        ))
        .add_plugins(HexMapNoisePlugin::<_, HexNoiseHeight>::new(
            Planet::default().with_seed(seed),
        ))
        .add_plugins(HexMapNoisePlugin::<_, HexNoiseTemperature>::new(
            PlanetTemperature::default().with_seed(seed + 1),
        ))
        .add_plugins(HexMapNoisePlugin::<_, HexNoiseHumidity>::new(
            PlanetHumidity::default().with_seed(seed + 2),
        ))
        .add_plugins(WASDCameraControllerPlugin)
        .add_plugins(DebugPlugin)
        .insert_resource(AssetsCache {
            layout,
            ..default()
        })
        .insert_resource(OverlayState::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (mouse_click_discover, input_switch_overlay, handle_hex),
        )
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

fn mouse_click_discover(
    windows: Query<&Window>,
    q_camera: Single<(&Camera, &GlobalTransform)>,
    mut ev_discover: EventWriter<HexDiscoverEvent>,
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

    ev_discover.write(HexDiscoverEvent(point.xz()));
}

fn input_switch_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut q_overlay: Query<(&mut Visibility, &MeshMaterial3d<HexMaterial>), With<OverlayMesh>>,
    mut q_tile: Query<&mut Visibility, (With<TileMesh>, Without<OverlayMesh>)>,
    mut materials: ResMut<Assets<HexMaterial>>,
    mut overlay_state: ResMut<OverlayState>,
) {
    if keys.just_pressed(KeyCode::ArrowUp) {
        overlay_state.kind = match overlay_state.kind {
            OverlayKind::Height => OverlayKind::Temperature,
            OverlayKind::Temperature => OverlayKind::Humidity,
            OverlayKind::Humidity => OverlayKind::Tile,
            OverlayKind::Tile => OverlayKind::Height,
        };

        if overlay_state.kind == OverlayKind::Tile {
            for (mut visibility, _) in q_overlay.iter_mut() {
                *visibility = Visibility::Hidden;
            }
            for mut visibility in q_tile.iter_mut() {
                *visibility = Visibility::Visible;
            }
        } else {
            for (mut visibility, material) in q_overlay.iter_mut() {
                if let Some(material) = materials.get_mut(material) {
                    material.mode = overlay_state.kind as u32;
                }
                *visibility = Visibility::Visible;
            }
            for mut visibility in q_tile.iter_mut() {
                *visibility = Visibility::Hidden;
            }
        }
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        overlay_state.kind = match overlay_state.kind {
            OverlayKind::Height => OverlayKind::Tile,
            OverlayKind::Temperature => OverlayKind::Height,
            OverlayKind::Humidity => OverlayKind::Temperature,
            OverlayKind::Tile => OverlayKind::Humidity,
        };

        if overlay_state.kind == OverlayKind::Tile {
            for (mut visibility, _) in q_overlay.iter_mut() {
                *visibility = Visibility::Hidden;
            }
            for mut visibility in q_tile.iter_mut() {
                *visibility = Visibility::Visible;
            }
        } else {
            for (mut visibility, material) in q_overlay.iter_mut() {
                if let Some(material) = materials.get_mut(material) {
                    material.mode = overlay_state.kind as u32;
                }
                *visibility = Visibility::Visible;
            }
            for mut visibility in q_tile.iter_mut() {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn handle_hex(
    mut commands: Commands,
    mut q_hex: Query<
        (
            Entity,
            &HexNoiseHeight,
            &HexNoiseTemperature,
            &HexNoiseHumidity,
            &mut Transform,
        ),
        (With<HexCoord>, Without<RenderedHex>),
    >,
    assets_cache: Res<AssetsCache>,
    mut materials: ResMut<Assets<HexMaterial>>,
    overlay_state: Res<OverlayState>,
) {
    for (
        entity,
        HexNoiseHeight(height),
        HexNoiseTemperature(temperature),
        HexNoiseHumidity(humidity),
        mut transform,
    ) in q_hex.iter_mut()
    {
        let tile = TileKind::from(*height);
        let temperature = (temperature * 10.0 - height * 2.0).clamp(-1.0, 1.0);
        let humidity = (humidity * 10.0 - height * 2.0).clamp(-1.0, 1.0);

        commands
            .entity(entity)
            .insert((Visibility::default(), RenderedHex))
            .with_children(|parent| {
                parent.spawn((
                    Name::new("Hex Overlay"),
                    Mesh3d(assets_cache.mesh.clone()),
                    MeshMaterial3d(materials.add(HexMaterial {
                        mode: overlay_state.kind as u32,
                        height: *height,
                        temperature: temperature,
                        humidity: humidity,
                        alpha_mode: AlphaMode::Opaque,
                    })),
                    OverlayMesh,
                    Visibility::Hidden,
                ));

                parent.spawn((
                    Name::new("Hex Tile"),
                    Mesh3d(assets_cache.mesh.clone()),
                    MeshMaterial3d(
                        assets_cache
                            .materials
                            .get(&tile)
                            .cloned()
                            .unwrap_or_default(),
                    ),
                    TileMesh,
                    Visibility::Visible,
                ));
            });

        transform.translation.y = *height * 5.0;
        if transform.translation.y < SEA_LEVEL as f32 {
            transform.translation.y = SEA_LEVEL as f32;
        }
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

/// Specifies the planet's sea level. This value must be between -1.0
/// (minimum planet elevation) and +1.0 (maximum planet elevation).
const SEA_LEVEL: f64 = 0.0;

#[derive(Clone, Copy, Debug)]
struct Planet {
    seed: u32,
    zoom_scale: f64,
    continent_frequency: f64,
    continent_lacunarity: f64,
    sea_level: f64,
}

impl Default for Planet {
    fn default() -> Self {
        Planet {
            seed: CURRENT_SEED,
            zoom_scale: ZOOM_SCALE,
            continent_frequency: CONTINENT_FREQUENCY,
            continent_lacunarity: CONTINENT_LACUNARITY,
            sea_level: SEA_LEVEL,
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
