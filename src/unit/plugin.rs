use std::collections::VecDeque;

use bevy::prelude::*;
use hexx::*;

use crate::{render::plugin::TileTopHeight, tilemap::hexmap::HexMapStorage};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnitPluginSet;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_pathfinding, handle_path_follow, handle_unit_render).in_set(UnitPluginSet),
        );
        app.add_systems(Update, handle_unit_placement.in_set(UnitPluginSet));
    }
}

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect, Deref, DerefMut)]
pub struct UnitCoord(pub Hex);

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect, Deref, DerefMut)]
pub struct UnitTarget(pub Hex);

#[derive(Component, Debug, Clone, Hash, PartialEq, Eq, Reflect, Deref, DerefMut)]
pub struct UnitPath(pub VecDeque<Hex>);

fn handle_pathfinding(
    mut commands: Commands,
    q_unit: Query<(Entity, &UnitCoord, &UnitTarget), Without<UnitPath>>,
    storage: Res<HexMapStorage>,
) {
    for (entity, coord, target) in q_unit.iter() {
        if coord.0 == target.0 {
            continue;
        }

        let path = storage
            .pathfinding(**coord, **target)
            .into_iter()
            .collect::<VecDeque<_>>();

        commands.entity(entity).insert(UnitPath(path));
    }
}

fn handle_path_follow(
    mut commands: Commands,
    mut q_unit: Query<(Entity, &mut UnitCoord, &mut UnitPath)>,
) {
    for (entity, mut coord, mut path) in q_unit.iter_mut() {
        if path.is_empty() {
            continue;
        }

        let next_hex = path.pop_front().unwrap();
        coord.0 = next_hex;

        if path.is_empty() {
            commands.entity(entity).remove::<UnitPath>();
        }
    }
}

#[derive(Component)]
struct UnitRenderReady;

fn handle_unit_render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_unit: Query<Entity, (With<UnitCoord>, Without<UnitRenderReady>)>,
) {
    for entity in q_unit.iter() {
        commands
            .entity(entity)
            .insert(UnitRenderReady)
            .with_children(|parent| {
                let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
                let material = materials.add(Color::srgb_u8(124, 144, 255));

                parent.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    Transform::IDENTITY,
                    Visibility::default(),
                    Name::new("Unit Render"),
                ));
            });
    }
}

// TODO: implement a glue module for stuff to interact between unit and hexmap

fn handle_unit_placement(
    mut q_unit: Query<(&mut Transform, &UnitCoord)>,
    q_hex: Query<(&GlobalTransform, &TileTopHeight), Without<UnitCoord>>,
    storage: Res<HexMapStorage>,
) {
    for (mut transform, coord) in q_unit.iter_mut() {
        if let Some(hex_entity) = storage.get_hex(**coord) {
            let Ok((hex_transform, TileTopHeight(height))) = q_hex.get(*hex_entity) else {
                continue;
            };

            transform.translation = hex_transform.translation() + Vec3::new(0.0, *height, 0.0);
        }
    }
}
