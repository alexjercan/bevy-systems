use avian3d::prelude::*;
use bevy::prelude::*;
use std::fmt::Debug;
use super::prelude::*;

pub mod prelude {
    pub use super::SectionMarker;
    pub use super::BaseSectionConfig;
    pub use super::SectionConfig;
    pub use super::SectionKind;
    pub use super::GameSections;
    pub use super::base_section;
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct SectionMarker;

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
    pub kind: SectionKind,
}

#[derive(Resource, Clone, Debug, Deref, DerefMut, Default)]
pub struct GameSections(pub Vec<SectionConfig>);

pub fn base_section(config: BaseSectionConfig) -> impl Bundle {
    debug!("Creating thruster section with config: {:?}", config);

    (
        Name::new(config.name.clone()),
        SectionMarker,
        // NOTE: Somehow I want to be able to use the mesh for the collider size later.
        Collider::cuboid(1.0, 1.0, 1.0),
        ColliderDensity(config.mass),
        Visibility::Inherited,
    )
}
