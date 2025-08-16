#[cfg(feature = "debug")]
use bevy::render::RenderSet;
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

use crate::{assets::prelude::*, terrain::prelude::*};

#[cfg(feature = "debug")]
use self::debug::{DebugPlugin, DebugSet};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderPluginSet;

pub struct RenderPlugin {
    layout: HexLayout,
    chunk_radius: u32,
    max_height: f32,
}

impl RenderPlugin {
    pub fn new(layout: HexLayout, chunk_radius: u32, max_height: f32) -> Self {
        Self {
            layout,
            chunk_radius,
            max_height,
        }
    }
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.add_plugins(DebugPlugin);
        #[cfg(feature = "debug")]
        app.configure_sets(Update, DebugSet.in_set(RenderPluginSet));

        app.insert_resource(HeightMapLayout::new(
            self.layout.clone(),
            self.chunk_radius,
            self.max_height,
        ))
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, ChunkMaterial>,
        >::default())
        .add_systems(
            Update,
            (
                handle_render_height,
                handle_feature_tile,
                handle_overlay_chunk,
            )
                .in_set(RenderPluginSet),
        );
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct HeightMapLayout {
    pub layout: HexLayout,
    pub chunk_radius: u32,
    pub max_height: f32,
}

impl HeightMapLayout {
    fn new(layout: HexLayout, chunk_radius: u32, max_height: f32) -> Self {
        Self {
            layout,
            chunk_radius,
            max_height,
        }
    }

    pub fn hexmap(&self, chunk: HashMap<Hex, f32>) -> Mesh {
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

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct TileTopHeight(pub f32);

fn handle_render_height(
    mut commands: Commands,
    q_hex: Query<(Entity, &HexNoiseHeight), Without<TileTopHeight>>,
    layout: Res<HeightMapLayout>,
) {
    if q_hex.is_empty() {
        return;
    }
    debug!("Handling tile top height for {} hexes", q_hex.iter().len());

    for (entity, height) in q_hex.iter() {
        let height = **height as f32;

        let height_value = (height * 2.0 - 1.0).clamp(0.0, 1.0);
        let height_mesh = (height_value * layout.max_height).round();

        commands.entity(entity).insert(TileTopHeight(height_mesh));
    }
}

#[derive(Component)]
struct ChunkFeatureReady;

fn handle_feature_tile(
    mut commands: Commands,
    assets: Res<MapAssets>,
    q_hex: Query<(Entity, &TileTopHeight, &HexTile, &HexFeature), Without<ChunkFeatureReady>>,
) {
    if q_hex.is_empty() {
        return;
    }
    debug!("Handling feature tiles for {} hexes", q_hex.iter().len());

    for (entity, height, tile, feature) in q_hex.iter() {
        commands
            .entity(entity)
            .insert(ChunkFeatureReady);

        let Some(id) = (**feature).clone() else {
            continue;
        };

        let Some(feature_asset) = assets.get_feature(&id) else {
            continue;
        };

        let Some(variant) = feature_asset.get_variant(&**tile) else {
            continue;
        };

        commands
            .entity(entity)
            .with_children(|parent| {
                parent.spawn((
                    Transform::from_xyz(0.0, **height, 0.0),
                    SceneRoot(variant.scene.clone()),
                    Name::new("Feature Tile"),
                ));
            });
    }
}

#[derive(Component)]
struct ChunkMeshReady;

fn handle_overlay_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ChunkMaterial>>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    layout: Res<HeightMapLayout>,
    q_hex: Query<(Entity, &HexCoord, &TileTopHeight, &HexTile, &ChildOf), Without<ChunkMeshReady>>,
    assets: Res<MapAssets>,
) {
    if q_hex.is_empty() {
        return;
    }
    debug!("Handling chunk mesh for {} hexes", q_hex.iter().len());

    let size = layout.chunk_radius * 2 + 1;
    for (&chunk_entity, chunk) in q_hex
        .iter()
        .chunk_by(|(_, _, _, _, ChildOf(e))| e)
        .into_iter()
    {
        let mut center: Option<Hex> = None;
        let mut storage = HashMap::default();
        let mut biome_data = vec![-1; (size * size) as usize];

        for (entity, coord, height, tile, _) in chunk {
            commands.entity(entity).insert(ChunkMeshReady);
            if center.is_none() {
                center = Some(
                    coord
                        .to_lower_res(layout.chunk_radius)
                        .to_higher_res(layout.chunk_radius),
                );
            }
            let coord = **coord - center.unwrap();

            storage.insert(coord, **height);

            let q_offset = coord.x + layout.chunk_radius as i32;
            let r_offset = coord.y + layout.chunk_radius as i32;
            let index = (r_offset * size as i32 + q_offset) as usize;
            biome_data[index] = assets.get_tile_index(&**tile).map_or(-1, |i| i as i32);
        }

        if let Some(center) = center {
            let mesh = layout.hexmap(storage);

            commands.entity(chunk_entity).with_children(|parent| {
                parent.spawn((
                    Visibility::Visible,
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
                            tiles: buffers.add(ShaderStorageBuffer::from(biome_data)),
                        },
                    })),
                    Name::new("Chunk Mesh"),
                ));
            });
        }
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
    pub tiles: Handle<ShaderStorageBuffer>,
}

impl MaterialExtension for ChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }
}

mod debug {
    use bevy::{
        pbr::{ExtendedMaterial, MaterialExtension},
        platform::collections::HashMap,
        prelude::*,
        render::{
            render_resource::{AsBindGroup, ShaderRef},
            storage::ShaderStorageBuffer,
        },
    };
    use hexx::*;
    use itertools::Itertools;

    use crate::{render::prelude::*, terrain::prelude::*};

    #[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
    pub(super) struct DebugSet;

    pub(super) struct DebugPlugin;

    impl Plugin for DebugPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(OverlayState::default())
                .add_plugins(MaterialPlugin::<
                    ExtendedMaterial<StandardMaterial, GradientMaterial>,
                >::default())
                .add_systems(
                    Update,
                    (
                        handle_overlay_chunk,
                        input_switch_overlay,
                        handle_overlay_update,
                    )
                        .in_set(DebugSet),
                );
        }
    }

    #[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect, Default)]
    enum OverlayKind {
        #[default]
        Tile,
        Height,
        Temperature,
        Humidity,
    }

    #[derive(Resource, Debug, Clone, Default)]
    struct OverlayState {
        kind: OverlayKind,
    }

    fn input_switch_overlay(
        keys: Res<ButtonInput<KeyCode>>,
        mut overlay_state: ResMut<OverlayState>,
    ) {
        if keys.just_pressed(KeyCode::ArrowUp) {
            overlay_state.kind = match overlay_state.kind {
                OverlayKind::Tile => OverlayKind::Height,
                OverlayKind::Height => OverlayKind::Temperature,
                OverlayKind::Temperature => OverlayKind::Humidity,
                OverlayKind::Humidity => OverlayKind::Tile,
            };
        } else if keys.just_pressed(KeyCode::ArrowDown) {
            overlay_state.kind = match overlay_state.kind {
                OverlayKind::Tile => OverlayKind::Humidity,
                OverlayKind::Height => OverlayKind::Tile,
                OverlayKind::Temperature => OverlayKind::Height,
                OverlayKind::Humidity => OverlayKind::Temperature,
            };
        }
    }

    #[derive(Component)]
    struct ChunkMeshReady;

    fn handle_overlay_chunk(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut gradient_materials: ResMut<
            Assets<ExtendedMaterial<StandardMaterial, GradientMaterial>>,
        >,
        mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
        layout: Res<HeightMapLayout>,
        q_hex: Query<
            (
                Entity,
                &HexCoord,
                &HexNoiseHeight,
                &HexNoiseHumidity,
                &HexNoiseTemperature,
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
            let mut height_data = vec![0.0; (size * size) as usize];
            let mut temperature_data = vec![0.0; (size * size) as usize];
            let mut humidity_data = vec![0.0; (size * size) as usize];

            for (entity, hex, noise, humidity, temperature, _) in chunk {
                commands.entity(entity).insert(ChunkMeshReady);
                let hex: Hex = **hex;
                if center.is_none() {
                    center = Some(
                        hex.to_lower_res(layout.chunk_radius)
                            .to_higher_res(layout.chunk_radius),
                    );
                }
                let hex = hex - center.unwrap();

                let height = **noise as f32;
                let humidity = **humidity as f32;
                let temperature = **temperature as f32;

                let height_value = height.clamp(0.0, 1.0);
                let height_mesh = (height_value * layout.max_height).round();
                storage.insert(hex, height_mesh);

                let temperature_value = ((temperature + 1.0) / 2.0).clamp(0.0, 1.0);
                let humidity_value = ((humidity + 1.0) / 2.0).clamp(0.0, 1.0);

                let q_offset = hex.x + layout.chunk_radius as i32;
                let r_offset = hex.y + layout.chunk_radius as i32;
                let index = (r_offset * size as i32 + q_offset) as usize;

                height_data[index] = height_value;
                temperature_data[index] = temperature_value;
                humidity_data[index] = humidity_value;
            }

            if let Some(center) = center {
                let mesh = layout.hexmap(storage);

                commands.entity(chunk_entity).with_children(|parent| {
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

    #[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
    pub(super) struct GradientMaterial {
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
}
