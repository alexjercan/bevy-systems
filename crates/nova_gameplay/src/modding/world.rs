use bevy::prelude::*;
use bevy_common_systems::prelude::EventWorld;

#[derive(Resource, Default)]
pub struct NovaEventWorld {}

impl EventWorld for NovaEventWorld {
    fn world_to_state_system(_world: &mut World) {}

    fn state_to_world_system(_world: &mut World) {}
}
