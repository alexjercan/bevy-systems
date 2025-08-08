//! TODO: Hexmap coordinates docs

#[path = "../helpers/wasd_camera_controller.rs"]
mod wasd_camera_controller;

#[path = "common.rs"]
mod common;

use bevy::{
    asset::RenderAssetUsages,
    color::palettes::css::GOLD,
    prelude::*,
    render::{
        camera::RenderTarget,
        mesh::{Indices, PrimitiveTopology},
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    },
};
use hexx::*;

use systems::{debug::DebugPlugin, hexmap::prelude::*};

use wasd_camera_controller::{WASDCameraControllerBundle, WASDCameraControllerPlugin};
use common::HexCoord;

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect, Default)]
enum OverlayKind {
    #[default]
    Offset,
    Axial,
}

#[derive(Resource, Debug, Clone, Default)]
struct OverlayState {
    kind: OverlayKind,
}

#[derive(Component, Debug, Clone, Copy)]
struct HexRendered;

const HEX_SIZE: f32 = 1.0;
const CHUNK_RADIUS: u32 = 4;
const DISCOVER_RADIUS: u32 = 0;

#[derive(Resource, Debug, Clone, Default)]
struct AssetsCache {
    mesh: Handle<Mesh>,
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
        .add_plugins(WASDCameraControllerPlugin)
        .add_plugins(DebugPlugin)
        .insert_resource(AssetsCache {
            layout,
            ..default()
        })
        .insert_resource(OverlayState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (input, input_switch_overlay, handle_hex))
        .configure_sets(Update, HexMapSet)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
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
}

fn input(
    windows: Query<&Window>,
    q_camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut ev_discover: EventWriter<HexDiscoverEvent::<HexCoord>>,
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

fn input_switch_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut q_overlay: Query<(&OverlayKind, &mut Visibility)>,
    mut overlay_state: ResMut<OverlayState>,
) {
    if keys.just_pressed(KeyCode::ArrowUp) {
        overlay_state.kind = match overlay_state.kind {
            OverlayKind::Offset => OverlayKind::Axial,
            OverlayKind::Axial => OverlayKind::Offset,
        };
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        overlay_state.kind = match overlay_state.kind {
            OverlayKind::Offset => OverlayKind::Axial,
            OverlayKind::Axial => OverlayKind::Offset,
        };
    }

    for (kind, mut visibility) in q_overlay.iter_mut() {
        *visibility = if *kind == overlay_state.kind {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn handle_hex(
    mut commands: Commands,
    q_hex: Query<(Entity, &HexCoord), Without<HexRendered>>,
    assets_cache: Res<AssetsCache>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, HexCoord(hex)) in q_hex.iter() {
        let size = Extent3d {
            width: 512,
            height: 512,
            ..default()
        };

        // This is the texture that will be rendered to.
        let mut image = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Bgra8UnormSrgb,
            RenderAssetUsages::default(),
        );
        // You need to set these texture usage flags in order to use the image as a render target
        image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT;

        let image_handle = images.add(image);

        let texture_camera = commands
            .spawn((
                Camera2d,
                Camera {
                    target: RenderTarget::Image(image_handle.clone().into()),
                    ..default()
                },
            ))
            .id();

        let offset = hex.to_offset_coordinates(OffsetHexMode::Even, HexOrientation::Flat);
        let axial = hex;

        commands
            .spawn((
                Node {
                    // Cover the whole image
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(GOLD.into()),
                UiTargetCamera(texture_camera),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(format!("{},{}", offset[0], offset[1])),
                    TextFont {
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor::BLACK,
                    Visibility::Visible,
                    OverlayKind::Offset,
                ));

                parent.spawn((
                    Text::new(format!("q{},r{}", axial.x, axial.y)),
                    TextFont {
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor::BLACK,
                    Visibility::Visible,
                    OverlayKind::Axial,
                ));
            });

        // This material has the texture that has been rendered.
        let material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(image_handle),
            reflectance: 0.02,
            unlit: false,

            ..default()
        });

        commands.entity(entity).insert((
            Mesh3d(assets_cache.mesh.clone()),
            MeshMaterial3d(material_handle),
            HexRendered,
        ));
    }
}
