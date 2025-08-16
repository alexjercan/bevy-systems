use super::{components::*, render::*};
use crate::{helpers::prelude::*, terrain::prelude::TileTopHeight};
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnitPluginSet;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenderPlugin)
            .configure_sets(Update, RenderPluginSet.in_set(UnitPluginSet))
            .add_systems(
                Update,
                (handle_pathfinding, handle_path_follow).in_set(UnitPluginSet),
            )
            .add_systems(Update, handle_unit_placement.in_set(UnitPluginSet));
    }
}

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

            println!("Placing unit at hex: {:?}", coord);

            transform.translation = hex_transform.translation() + Vec3::new(0.0, *height, 0.0);
        }
    }
}
