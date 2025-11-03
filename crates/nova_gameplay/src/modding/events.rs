use bevy::prelude::*;
use bevy_common_systems::prelude::EventKind;

pub mod prelude {
    pub use super::{OnDestroyedEvent, OnDestroyedEventInfo, OnStartEvent, OnStartEventInfo};
}

#[derive(Debug, Clone, EventKind, Reflect)]
#[event_name("onstart")]
#[event_info(OnStartEventInfo)]
pub struct OnStartEvent;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, Reflect)]
pub struct OnStartEventInfo;

#[derive(Debug, Clone, EventKind, Reflect)]
#[event_name("ondestroyed")]
#[event_info(OnDestroyedEventInfo)]
pub struct OnDestroyedEvent;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, Reflect)]
pub struct OnDestroyedEventInfo {
    pub id: String,
    pub type_name: String,
}
