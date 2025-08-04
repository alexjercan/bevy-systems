//! TODO: Hexmap coordinates docs

use std::time::SystemTime;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use hexx::*;
use leafwing_input_manager::prelude::*;

use systems::{
    camera::rts_camera::{RTSCamera, RTSCameraInput, RTSCameraPlugin, RTSCameraSet},
    hexmap::perlin::{HexCoord, HexMapPerlinPlugin, HexMapPerlinSet, HexNoise, HexProbe},
    debug::DebugPlugin,
};

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum CameraMovement {
    #[actionlike(Axis)]
    Zoom,
    #[actionlike(DualAxis)]
    Pan,
    HoldPan,
    HoldOrbit,
}

const CHUNK_RADIUS: u32 = 4;
const HEX_SIZE: f32 = 1.0;
const COLUMN_HEIGHT: f32 = 10.0;

// These could be read from a config using bevy_asset_loader
const COLORS: [Color; 6] = [
    Color::srgb_u8(255, 255, 255), // Mountains
    Color::srgb_u8(139, 69, 19),   // Hills
    Color::srgb_u8(0, 128, 0),     // Plains
    Color::srgb_u8(255, 255, 0),   // Sand
    Color::srgb_u8(0, 0, 255),     // Water
    Color::srgb_u8(0, 0, 139),     // Deep Water
];
const HEIGHTS: [f32; 6] = [
    1.0,  // Mountains
    0.6,  // Hills
    0.3,  // Plains
    0.1,  // Sand
    0.0,  // Water
    -0.5, // Deep Water
];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HexMapPerlinPlugin::new(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32,
            HEX_SIZE,
            CHUNK_RADIUS,
        ))
        .add_plugins(RTSCameraPlugin)
        .add_plugins(InputManagerPlugin::<CameraMovement>::default())
        .add_plugins(DebugPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (input, handle_hex))
        .configure_sets(Update, RTSCameraSet)
        .configure_sets(Update, HexMapPerlinSet)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        RTSCamera::default(),
        RTSCameraInput::default(),
        InputMap::default()
            .with_axis(CameraMovement::Zoom, MouseScrollAxis::Y)
            .with_dual_axis(CameraMovement::Pan, MouseMove::default())
            .with(CameraMovement::HoldOrbit, MouseButton::Right)
            .with(CameraMovement::HoldPan, MouseButton::Middle),
        Camera3d::default(),
        Transform::from_xyz(60.0, 60.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("RTS Camera"),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(60.0, 60.0, 00.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Directional Light"),
    ));

    commands.spawn((
        HexProbe,
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("Hex Probe"),
    ));
}

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

fn input(mut q_camera: Query<(&mut RTSCameraInput, &ActionState<CameraMovement>)>) {
    for (mut input, action) in q_camera.iter_mut() {
        input.pan = Vec2::ZERO;
        input.orbit = Vec2::ZERO;

        if action.pressed(&CameraMovement::HoldOrbit) {
            input.orbit = action.axis_pair(&CameraMovement::Pan);
        } else if action.pressed(&CameraMovement::HoldPan) {
            input.pan = action.axis_pair(&CameraMovement::Pan);
        }

        input.zoom = action.value(&CameraMovement::Zoom);
    }
}

fn handle_hex(
    mut commands: Commands,
    q_hex: Query<(Entity, &HexNoise, &Transform), (With<HexCoord>, Without<Mesh3d>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, HexNoise(height), transform) in q_hex.iter() {
        // TODO: how to handle this better?
        let layout = HexLayout {
            scale: Vec2::splat(HEX_SIZE),
            ..default()
        };

        let height_index = HEIGHTS
            .iter()
            .rposition(|&h| *height <= h)
            .unwrap_or(HEIGHTS.len() - 1);

        let color = COLORS[height_index];
        // let height_value = height.clamp(0.0, 1.0); // water has level 0
        // let height_value = height_value as f32 * COLUMN_HEIGHT;

        commands.entity(entity).insert((
            Mesh3d(meshes.add(hexagonal_column(&layout))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 1.0,
                metallic: 0.0,
                ..default()
            })),
        ));
    }
}
