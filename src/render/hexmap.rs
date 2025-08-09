use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        storage::ShaderStorageBuffer,
    },
};
use hexx::*;
use itertools::Itertools;

pub mod prelude {
    pub use super::{HexMapMaterialPlugin, HexMapMaterialSet};
}

#[derive(Resource, Debug, Clone, Default)]
struct Layout {
    layout: HexLayout,
    chunk_radius: u32,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HexMapMaterialSet;

pub struct HexMapMaterialPlugin<T, C> {
    layout: HexLayout,
    chunk_radius: u32,
    _marker_in: std::marker::PhantomData<T>,
    _marker_out: std::marker::PhantomData<C>,
}

impl<T, C> HexMapMaterialPlugin<T, C> {
    pub fn new(layout: HexLayout, chunk_radius: u32) -> Self {
        Self {
            layout,
            chunk_radius,
            _marker_in: std::marker::PhantomData,
            _marker_out: std::marker::PhantomData,
        }
    }
}

impl<T: Component + Send + Sync + 'static, C: Component + Send + Sync + 'static> Plugin
    for HexMapMaterialPlugin<T, C>
where
    for<'a> &'a T: Into<Hex>,
    for<'a> &'a C: Into<LinearRgba>,
{
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, ChunkMaterial>,
        >::default());

        app.insert_resource(Layout {
            layout: self.layout.clone(),
            chunk_radius: self.chunk_radius,
        });

        app.add_systems(Update, (handle_hex::<T, C>).in_set(HexMapMaterialSet));
    }
}

#[derive(Component)]
struct ChunkMaterialReady;

fn handle_hex<T: Component + Send + Sync + 'static, C: Component + Send + Sync + 'static>(
    mut commands: Commands,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ChunkMaterial>>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    layout: Res<Layout>,
    q_hex: Query<(Entity, &T, &C, &ChildOf), Without<ChunkMaterialReady>>,
) where
    for<'a> &'a T: Into<Hex>,
    for<'a> &'a C: Into<LinearRgba>,
{
    for (&chunk_entity, chunk) in q_hex.iter().chunk_by(|(_, _, _, ChildOf(e))| e).into_iter() {
        let mut center: Option<Hex> = None;
        let size = layout.chunk_radius * 2 + 1;
        let mut color_data = vec![LinearRgba::NONE; (size * size) as usize];

        for (entity, hex, noise, _) in chunk {
            commands.entity(entity).insert(ChunkMaterialReady);
            let hex: Hex = hex.into();
            if center.is_none() {
                center = Some(
                    hex.to_lower_res(layout.chunk_radius)
                        .to_higher_res(layout.chunk_radius),
                );
            }
            let hex = hex - center.unwrap();

            let q_offset = hex.x + layout.chunk_radius as i32;
            let r_offset = hex.y + layout.chunk_radius as i32;
            let index = (r_offset * size as i32 + q_offset) as usize;
            color_data[index] = noise.into();
        }

        if let Some(center) = center {
            commands
                .entity(chunk_entity)
                .insert((MeshMaterial3d(materials.add(ExtendedMaterial {
                    base: StandardMaterial {
                        perceptual_roughness: 1.0,
                        metallic: 0.0,
                        ..default()
                    },
                    extension: ChunkMaterial {
                        chunk_radius: layout.chunk_radius,
                        hex_size: layout.layout.scale.x,
                        chunk_center: IVec2::new(center.x, center.y),
                        noise: buffers.add(ShaderStorageBuffer::from(color_data)),
                    },
                })),));
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
    pub noise: Handle<ShaderStorageBuffer>,
}

impl MaterialExtension for ChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }
}
