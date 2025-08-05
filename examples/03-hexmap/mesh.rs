//! TODO: Hexmap coordinates docs with simple mesh

use std::time::SystemTime;

use hexx::*;
use bevy::{asset::RenderAssetUsages, prelude::*, render::mesh::{Indices, PrimitiveTopology}};
use leafwing_input_manager::prelude::*;

use systems::{
    camera::wasd_camera::{WASDCamera, WASDCameraInput, WASDCameraPlugin, WASDCameraSet},
    debug::DebugPlugin,
    hexmap::map::{GeneratorKind, HexCoord, HexDiscoverEvent, HexMapPlugin, HexMapSet, HexNoise},
};

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum CameraMovement {
    #[actionlike(DualAxis)]
    Pan,
    #[actionlike(DualAxis)]
    Wasd,
    #[actionlike(Axis)]
    Vertical,
    HoldPan,
    LeftClick,
}

const CHUNK_RADIUS: u32 = 4;
const HEX_SIZE: f32 = 1.0;
const DISCOVER_RADIUS: u32 = 3;
const COLUMN_HEIGHT: f32 = 5.0;

fn hexagonal_column(hex_layout: &HexLayout) -> Mesh {
    let mesh_info = ColumnMeshBuilder::new(hex_layout, COLUMN_HEIGHT)
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HexMapPlugin::new(
            GeneratorKind::Perlin(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u32,
            ),
            HEX_SIZE,
            CHUNK_RADIUS,
            DISCOVER_RADIUS,
        ))
        .add_plugins(WASDCameraPlugin)
        .add_plugins(InputManagerPlugin::<CameraMovement>::default())
        .add_plugins(DebugPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (input, handle_hex))
        .configure_sets(Update, WASDCameraSet)
        .configure_sets(Update, HexMapSet)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        WASDCamera::default(),
        WASDCameraInput::default(),
        InputMap::default()
            .with_dual_axis(CameraMovement::Pan, MouseMove::default())
            .with_dual_axis(CameraMovement::Wasd, VirtualDPad::wasd())
            .with_axis(
                CameraMovement::Vertical,
                VirtualAxis::new(KeyCode::ShiftLeft, KeyCode::Space),
            )
            .with(CameraMovement::HoldPan, MouseButton::Right)
            .with(CameraMovement::LeftClick, MouseButton::Left),
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
    mut q_camera: Query<(
        &Camera,
        &GlobalTransform,
        &mut WASDCameraInput,
        &ActionState<CameraMovement>,
    )>,
    mut ev_discover: EventWriter<HexDiscoverEvent>,
) {
    for (camera, transform, mut input, action) in q_camera.iter_mut() {
        input.pan = Vec2::ZERO;

        if action.pressed(&CameraMovement::HoldPan) {
            input.pan = action.axis_pair(&CameraMovement::Pan);
        }

        input.wasd = action.axis_pair(&CameraMovement::Wasd);
        input.vertical = action.value(&CameraMovement::Vertical);

        if action.just_pressed(&CameraMovement::LeftClick) {
            let Some(cursor_position) = windows.single().unwrap().cursor_position() else {
                return;
            };

            let Ok(ray) = camera.viewport_to_world(transform, cursor_position) else {
                return;
            };

            let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
            else {
                return;
            };
            let point = ray.get_point(distance);

            ev_discover.write(HexDiscoverEvent(point.xz()));
        }
    }
}

fn handle_hex(
    mut commands: Commands,
    mut q_hex: Query<(Entity, &HexNoise, &mut Transform), (With<HexCoord>, Without<Mesh3d>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, HexNoise(height), mut transform) in q_hex.iter_mut() {
        let layout = HexLayout {
            scale: Vec2::splat(HEX_SIZE),
            ..default()
        };

        let tile = TileKind::from(*height);
        let color = tile.into();
        let height_value = height.clamp(0.0, 1.0);
        let height_value = height_value as f32 * COLUMN_HEIGHT;

        commands.entity(entity).insert((
            Mesh3d(meshes.add(hexagonal_column(&layout))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 1.0,
                metallic: 0.0,
                ..default()
            })),
        ));

        transform.translation.y = height_value;
    }
}
