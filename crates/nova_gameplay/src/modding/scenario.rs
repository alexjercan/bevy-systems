use bevy::prelude::*;
use crate::prelude::SectionConfig;

use super::filters::EventFilterConfig;
use super::actions::EventActionConfig;

pub mod prelude {
    pub use super::{GameScenarios, MapConfig, ScenarioConfig, GameObjectConfig, AsteroidConfig, SpaceshipConfig, SpaceshipSectionConfig, ScenarioEventConfig};
}

#[derive(Resource, Clone, Debug, Deref, DerefMut, Default)]
pub struct GameScenarios(pub Vec<ScenarioConfig>);

#[derive(Clone, Debug)]
pub struct ScenarioConfig {
    pub name: String,
    pub description: String,
    pub map: MapConfig,
    pub events: Vec<ScenarioEventConfig>,
}

#[derive(Clone, Debug)]
pub struct MapConfig {
    pub objects: Vec<GameObjectConfig>,
}

#[derive(Clone, Debug)]
pub enum GameObjectConfig {
    Asteroid(AsteroidConfig),
    Spaceship(SpaceshipConfig),
}

#[derive(Clone, Debug)]
pub struct AsteroidConfig {
    pub id: String,
    pub name: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub radius: f32,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub struct SpaceshipConfig {
    pub id: String,
    pub name: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub sections: Vec<SpaceshipSectionConfig>,
}

#[derive(Clone, Debug)]
pub struct SpaceshipSectionConfig {
    pub position: Vec3,
    pub rotation: Quat,
    // NOTE: Maybe in the future this will be a Handle and in the .cfg file it will be represented
    // by an ID.
    pub config: SectionConfig,
}

#[derive(Clone, Debug)]
pub struct ScenarioEventConfig {
    pub name: String,
    pub filters: Vec<EventFilterConfig>,
    pub actions: Vec<EventActionConfig>,
}
