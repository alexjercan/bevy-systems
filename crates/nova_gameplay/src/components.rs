use bevy::prelude::*;

pub mod prelude {
    pub use super::{EntityId, EntityTypeName};
}

#[derive(Component, Debug, Clone, Default, Deref, DerefMut, Reflect)]
pub struct EntityId(pub String);

impl EntityId {
    pub fn new<S: Into<String>>(s: S) -> Self {
        EntityId(s.into())
    }
}

#[derive(Component, Debug, Clone, Default, Deref, DerefMut, Reflect)]
pub struct EntityTypeName(pub String);

impl EntityTypeName {
    pub fn new<S: Into<String>>(s: S) -> Self {
        EntityTypeName(s.into())
    }
}
