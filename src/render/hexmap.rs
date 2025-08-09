use bevy::{
    asset::RenderAssetUsages,
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

pub mod prelude {
    pub use super::{RenderPlugin, RenderSet};
}

#[derive(Resource, Debug, Clone, Default)]
struct Layout {
    layout: HexLayout,
    chunk_radius: u32,
    max_height: f32,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderSet;

pub struct RenderPlugin<T, C> {
    layout: HexLayout,
    chunk_radius: u32,
    max_height: f32,
    _marker_in: std::marker::PhantomData<T>,
    _marker_out: std::marker::PhantomData<C>,
}

impl<T, C> RenderPlugin<T, C> {
    pub fn new(layout: HexLayout, chunk_radius: u32, max_height: f32) -> Self {
        Self {
            layout,
            chunk_radius,
            max_height,
            _marker_in: std::marker::PhantomData,
            _marker_out: std::marker::PhantomData,
        }
    }
}

impl<T: Component + Send + Sync + 'static, C: Component + Send + Sync + 'static> Plugin
    for RenderPlugin<T, C>
where
    for<'a> &'a T: Into<Hex>,
    for<'a> &'a C: Into<f64>,
{
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ChunkMaterial>::default());

        app.insert_resource(Layout {
            layout: self.layout.clone(),
            chunk_radius: self.chunk_radius,
            max_height: self.max_height,
        });

        app.add_systems(Update, (handle_hex::<T, C>).in_set(RenderSet));
    }
}

impl Layout {
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
struct RenderHex;

fn handle_hex<T: Component + Send + Sync + 'static, C: Component + Send + Sync + 'static>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    layout: Res<Layout>,
    q_hex: Query<(Entity, &T, &C, &ChildOf), Without<RenderHex>>,
) where
    for<'a> &'a T: Into<Hex>,
    for<'a> &'a C: Into<f64>,
{
    for (&chunk_entity, chunk) in q_hex.iter().chunk_by(|(_, _, _, ChildOf(e))| e).into_iter() {
        let mut storage = HashMap::default();

        let mut center: Option<Hex> = None;
        let size = layout.chunk_radius * 2 + 1;
        let mut noise_data = vec![0.0; (size * size) as usize];

        for (entity, hex, noise, _) in chunk {
            commands.entity(entity).insert(RenderHex);
            let hex = hex.into();
            if center.is_none() {
                center = Some(
                    hex.to_lower_res(layout.chunk_radius)
                        .to_higher_res(layout.chunk_radius),
                );
            }
            let hex = hex - center.unwrap();

            storage.insert(
                hex,
                (noise.into() as f32).clamp(0.0, 1.0) * layout.max_height,
            );

            let q_offset = hex.x + layout.chunk_radius as i32;
            let r_offset = hex.y + layout.chunk_radius as i32;
            let index = (r_offset * size as i32 + q_offset) as usize;
            noise_data[index] = noise.into() as f32;
        }

        if let Some(center) = center {
            let mesh = layout.hexmap(storage);

            commands.entity(chunk_entity).insert((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(ChunkMaterial {
                    chunk_radius: layout.chunk_radius,
                    hex_size: layout.layout.scale.x,
                    chunk_center: IVec2::new(center.x, center.y),
                    noise: buffers.add(ShaderStorageBuffer::from(noise_data)),
                    alpha_mode: AlphaMode::Opaque,
                })),
            ));
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {
    #[uniform(0)]
    pub chunk_radius: u32,
    #[uniform(1)]
    pub hex_size: f32,
    #[uniform(2)]
    pub chunk_center: IVec2,
    #[storage(3, read_only)]
    pub noise: Handle<ShaderStorageBuffer>,

    alpha_mode: AlphaMode,
}

impl Material for ChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}
