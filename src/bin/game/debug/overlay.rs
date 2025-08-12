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
pub struct OverlayPluginSet;

pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
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
                    .in_set(OverlayPluginSet),
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

fn input_switch_overlay(keys: Res<ButtonInput<KeyCode>>, mut overlay_state: ResMut<OverlayState>) {
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
    mut gradient_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, GradientMaterial>>>,
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
            let hex: Hex = hex.into();
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
pub struct GradientMaterial {
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
