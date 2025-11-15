use bevy::prelude::*;
use bevy_common_systems::prelude::*;

pub mod prelude {
    pub use super::{
        EntityId, EntityTypeName, OnDestroyedEvent, OnDestroyedEventInfo, OnStartEvent,
        OnStartEventInfo, OnUpdateEvent, OnUpdateEventInfo, ENTITY_ID_COMPONENT_NAME,
        ENTITY_TYPE_NAME_COMPONENT_NAME,
    };
}

#[derive(Component, Debug, Clone, Default, Deref, DerefMut, Reflect)]
pub struct EntityId(pub String);

impl EntityId {
    pub fn new<S: Into<String>>(s: S) -> Self {
        EntityId(s.into())
    }
}

pub const ENTITY_ID_COMPONENT_NAME: &str = "id";

#[derive(Component, Debug, Clone, Default, Deref, DerefMut, Reflect)]
pub struct EntityTypeName(pub String);

impl EntityTypeName {
    pub fn new<S: Into<String>>(s: S) -> Self {
        EntityTypeName(s.into())
    }
}

pub const ENTITY_TYPE_NAME_COMPONENT_NAME: &str = "type_name";

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
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "type_name")]
    pub type_name: String,
}

#[derive(Debug, Clone, EventKind, Reflect)]
#[event_name("onupdate")]
#[event_info(OnUpdateEventInfo)]
pub struct OnUpdateEvent;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, Reflect)]
pub struct OnUpdateEventInfo;
