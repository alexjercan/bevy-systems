use bevy::{
    asset::RenderAssetUsages,
    platform::collections::HashMap,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
    },
};
use hexx::*;
use itertools::Itertools;

pub mod prelude {
    pub use super::{HexMapMeshPlugin, HexMapMeshSet};
}

#[derive(Resource, Debug, Clone, Default)]
struct Layout {
    layout: HexLayout,
    chunk_radius: u32,
    max_height: f32,
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

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HexMapMeshSet;

pub struct HexMapMeshPlugin<T, C> {
    layout: HexLayout,
    chunk_radius: u32,
    max_height: f32,
    _marker_in: std::marker::PhantomData<T>,
    _marker_out: std::marker::PhantomData<C>,
}

impl<T, C> HexMapMeshPlugin<T, C> {
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
    for HexMapMeshPlugin<T, C>
where
    for<'a> &'a T: Into<Hex>,
    for<'a> &'a C: Into<f64>,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(Layout {
            layout: self.layout.clone(),
            chunk_radius: self.chunk_radius,
            max_height: self.max_height,
        });

        app.add_systems(Update, (handle_hex::<T, C>).in_set(HexMapMeshSet));
    }
}

#[derive(Component)]
struct ChunkMeshReady;

fn handle_hex<T: Component + Send + Sync + 'static, C: Component + Send + Sync + 'static>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    layout: Res<Layout>,
    q_hex: Query<(Entity, &T, &C, &ChildOf), Without<ChunkMeshReady>>,
) where
    for<'a> &'a T: Into<Hex>,
    for<'a> &'a C: Into<f64>,
{
    for (&chunk_entity, chunk) in q_hex.iter().chunk_by(|(_, _, _, ChildOf(e))| e).into_iter() {
        let mut storage = HashMap::default();
        let mut center: Option<Hex> = None;

        for (entity, hex, noise, _) in chunk {
            commands.entity(entity).insert(ChunkMeshReady);
            let hex: Hex = hex.into();
            if center.is_none() {
                center = Some(
                    hex.to_lower_res(layout.chunk_radius)
                        .to_higher_res(layout.chunk_radius),
                );
            }
            let hex = hex - center.unwrap();

            let noise_value: f64 = noise.into();
            storage.insert(hex, noise_value.clamp(0.0, 1.0) as f32 * layout.max_height);
        }

        let mesh = layout.hexmap(storage);

        commands
            .entity(chunk_entity)
            .insert(Mesh3d(meshes.add(mesh)));
    }
}
