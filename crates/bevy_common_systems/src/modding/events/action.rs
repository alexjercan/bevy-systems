use super::game_event::GameEventInfo;
use bevy::prelude::*;

pub trait EventAction: std::fmt::Debug + Send + Sync {
    fn action(&self, commands: &mut Commands, info: &GameEventInfo);

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
