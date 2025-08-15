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

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
struct TileTopHeight(f32);

fn handle_render_height(
    mut commands: Commands,
    q_hex: Query<(Entity, &HexNoiseHeight), Without<TileTopHeight>>,
    layout: Res<HeightMapLayout>,
) {
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
    feature_assets: Res<Assets<FeatureAsset>>,
    game_assets: Res<GameAssets>,
    q_hex: Query<(Entity, &TileTopHeight, &HexFeature), Without<ChunkFeatureReady>>,
) {
    for (entity, height, feature) in q_hex.iter() {
        let index = **feature;
        if index < 0 {
            continue;
        }

        if let Some(feature_asset) = feature_assets.get(&game_assets.features[index as usize]) {
            commands
                .entity(entity)
                .insert(ChunkFeatureReady)
                .with_children(|parent| {
                    parent.spawn((
                        Transform::from_xyz(0.0, **height, 0.0),
                        SceneRoot(feature_asset.scene.clone()),
                        Name::new("Feature Tile"),
                    ));
                });
        }
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
) {
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
            biome_data[index] = **tile;
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
