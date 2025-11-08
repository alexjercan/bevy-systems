use avian3d::prelude::*;
use bevy::{mesh::{Indices, VertexAttributeValues}, prelude::*};
use bevy_common_systems::prelude::*;
use clap::Parser;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

#[derive(Parser)]
#[command(name = "02_sphere")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how to create a basic sphere and then apply noise in bevy_common_systems", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(PhysicsPlugins::default());

    #[cfg(feature = "debug")]
    app.add_plugins(InpsectorDebugPlugin);

    app.add_plugins(bevy_enhanced_input::EnhancedInputPlugin);
    app.add_plugins(WASDCameraPlugin);
    app.add_plugins(WASDCameraControllerPlugin);

    app.add_systems(Startup, setup);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = apply_noise_to_mesh(&octahedron_sphere(3));

    commands.spawn((
        Name::new("Sphere"),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.7, 0.6),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));

    commands.spawn((
        Name::new("Main Camera"),
        Camera3d::default(),
        WASDCameraController,
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_2,
            0.0,
            0.0,
        )),
        GlobalTransform::default(),
    ));
}

pub fn apply_noise_to_mesh(mesh: &Mesh) -> Mesh {
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
        VertexAttributeValues::Float32x3(vals) => {
            vals.iter().map(|v| Vec3::from(*v)).collect::<Vec<_>>()
        }
        _ => panic!("Unsupported position format"),
    };

    let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap() {
        VertexAttributeValues::Float32x3(vals) => {
            vals.iter().map(|v| Vec3::from(*v)).collect::<Vec<_>>()
        }
        _ => vec![Vec3::ZERO; positions.len()],
    };

    let planet = PlanetHeight::default();
    let height_values = positions.iter().map(|&p| planet.get(p)).collect::<Vec<_>>();

    let positions = positions
        .iter()
        .zip(normals.iter())
        .zip(height_values.iter())
        .map(|((pos, normal), height)| pos + normal * (*height as f32 * 5.0))
        .collect::<Vec<_>>();

    let triangles = match mesh.indices().unwrap() {
        Indices::U32(indices) => indices.to_vec(),
        Indices::U16(indices) => indices.iter().map(|&i| i as u32).collect::<Vec<_>>(),
    }
    .chunks(3)
    .map(|c| {
        Triangle3d::new(
            positions[c[0] as usize],
            positions[c[1] as usize],
            positions[c[2] as usize],
        )
    })
    .collect::<Vec<_>>();

    let builder = TriangleMeshBuilder { triangles };

    builder.build()
}

/// Planet seed. Change this to generate a different planet.
const CURRENT_SEED: u32 = 0;

/// Scale of the planet. Change this to zoom in or out.
const ZOOM_SCALE: f64 = 0.001;

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

#[derive(Resource, Clone, Copy, Debug)]
pub struct PlanetHeight {
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
    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }

    pub fn get(&self, point: Vec3) -> f64 {
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

        let x = point.x as f64 * self.zoom_scale;
        let y = point.y as f64 * self.zoom_scale;

        let noise = base_continent_def.get([x, y]);
        (noise + 1.0) * 0.5
    }
}
