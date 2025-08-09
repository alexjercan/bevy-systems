//! TODO: Hexmap coordinates docs

#[path = "../helpers/wasd_camera_controller.rs"]
mod wasd_camera_controller;

#[path = "common.rs"]
mod common;

use bevy::prelude::*;
use hexx::*;

use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use systems::{
    debug::DebugPlugin, hexmap::prelude::*, mesh::prelude::*, noise::prelude::*, render::prelude::*,
};

use common::HexCoord;
use wasd_camera_controller::{WASDCameraControllerBundle, WASDCameraControllerPlugin};

const HEX_SIZE: f32 = 1.0;
const CHUNK_RADIUS: u32 = 15;
const DISCOVER_RADIUS: u32 = 3;
const COLUMN_HEIGHT: f32 = 5.0;

#[derive(Component, Debug, Clone, Copy)]
struct HexNoise(f64);

impl From<f64> for HexNoise {
    fn from(noise: f64) -> Self {
        Self(noise)
    }
}

impl Into<f64> for &HexNoise {
    fn into(self) -> f64 {
        self.0
    }
}

impl Into<LinearRgba> for &HexNoise {
    fn into(self) -> LinearRgba {
        let value = self.0.clamp(-1.0, 1.0) as f32;
        if value <= -0.5 {
            LinearRgba::new(0.0, 0.0, 139.0 / 255.0, 1.0) // Deep Water
        } else if value <= 0.0 {
            LinearRgba::new(0.0, 0.0, 1.0, 1.0) // Water
        } else if value <= 0.1 {
            LinearRgba::new(1.0, 1.0, 0.0, 1.0) // Sand
        } else if value <= 0.3 {
            LinearRgba::new(0.0, 128.0 / 255.0, 0.0, 1.0) // Grass
        } else if value <= 0.6 {
            LinearRgba::new(139.0 / 255.0, 69.0 / 255.0, 19.0 / 255.0, 1.0) // Hills
        } else {
            LinearRgba::new(1.0, 1.0, 1.0, 1.0) // Mountains
        }
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
        .add_plugins(HexMapMeshPlugin::<HexCoord, HexNoise>::new(
            layout.clone(),
            CHUNK_RADIUS,
            COLUMN_HEIGHT,
        ))
        .add_plugins(HexMapMaterialPlugin::<HexCoord, HexNoise>::new(
            layout.clone(),
            CHUNK_RADIUS,
        ))
        .add_plugins(WASDCameraControllerPlugin)
        .add_plugins(DebugPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, input)
        .configure_sets(Update, HexMapSet)
        .configure_sets(Update, NoiseSet)
        .configure_sets(Update, HexMapMeshSet)
        .configure_sets(Update, HexMapMaterialSet)
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
