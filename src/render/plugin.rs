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

use crate::{
    assets::prelude::*,
    terrain::prelude::*,
};

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
        .add_systems(Update, handle_overlay_chunk.in_set(RenderPluginSet));
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
    terrain: Res<Assets<TileAsset>>,
    assets: Res<GameAssets>,
) {
    let size = layout.chunk_radius * 2 + 1;
    for (&chunk_entity, chunk) in q_hex
        .iter()
        .chunk_by(|(_, _, _, _, _, ChildOf(e))| e)
        .into_iter()
    {
        let mut center: Option<Hex> = None;
        let mut storage = HashMap::default();
        let mut biome_data = vec![-1; (size * size) as usize];

        for (entity, hex, height, humidity, temperature, _) in chunk {
            commands.entity(entity).insert(ChunkMeshReady);
            let hex: Hex = **hex;
            if center.is_none() {
                center = Some(
                    hex.to_lower_res(layout.chunk_radius)
                        .to_higher_res(layout.chunk_radius),
                );
            }
            let hex = hex - center.unwrap();

            let height = **height as f32;
            let humidity = **humidity as f32;
            let temperature = **temperature as f32;

            let height_value = height.clamp(0.0, 1.0);
            let height_mesh = (height_value * layout.max_height).round();
            storage.insert(hex, height_mesh);

            let biome = assets
                .terrain_index(&terrain, height, humidity, temperature)
                .map_or(-1, |v| v as i32);

            let q_offset = hex.x + layout.chunk_radius as i32;
            let r_offset = hex.y + layout.chunk_radius as i32;
            let index = (r_offset * size as i32 + q_offset) as usize;
            biome_data[index] = biome as i32;
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
