use std::fmt::Debug;

use avian3d::prelude::*;
use bevy::prelude::*;

use super::prelude::*;

pub mod prelude {
    pub use super::{
        base_section, BaseSectionConfig, GameSections, SectionConfig, SectionKind, SectionMarker,
        SectionRenderOf,
    };
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct SectionMarker;

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect, PartialEq, Eq)]
pub struct SectionRenderOf(pub Entity);

#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct BaseSectionConfig {
    pub name: String,
    pub description: String,
    pub mass: f32,
}

#[derive(Clone, Debug)]
pub enum SectionKind {
    Hull(HullSectionConfig),
    Thruster(ThrusterSectionConfig),
    Controller(ControllerSectionConfig),
    Turret(TurretSectionConfig),
}

#[derive(Clone, Debug)]
pub struct SectionConfig {
    pub base: BaseSectionConfig,
    // TODO: I will probably want to box this later to reduce the size of the struct.
    pub kind: SectionKind,
}

#[derive(Resource, Clone, Debug, Deref, DerefMut, Default)]
pub struct GameSections(pub Vec<SectionConfig>);

pub fn base_section(config: BaseSectionConfig) -> impl Bundle {
    debug!("base_section: config {:?}", config);

    (
        Name::new(config.name.clone()),
        SectionMarker,
        // NOTE: Somehow I want to be able to use the mesh for the collider size later.
        // Ideally I will not use the mesh for collider because that will be expensive.
        // But I want to parametrise the collider to look better than just a cube.
        Collider::cuboid(1.0, 1.0, 1.0),
        ColliderDensity(config.mass),
        Visibility::Inherited,
    )
}
