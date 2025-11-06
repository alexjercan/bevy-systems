//! Gameplay related functionality for Nova Protocol.
//!
//! Nova Protocol specific systems and components.

pub mod camera_controller;
pub mod components;
pub mod damage;
pub mod hud;
pub mod input;
pub mod modding;
pub mod plugin;
pub mod sections;

pub use bevy_common_systems;

pub mod prelude {
    // Re-export bevy_common_systems prelude
    pub use bevy_common_systems::prelude::*;

    pub use super::{
        camera_controller::prelude::*,
        components::prelude::*,
        damage::prelude::*,
        hud::prelude::*,
        input::prelude::*,
        modding::prelude::*,
        plugin::{NovaGameplayPlugin, SpaceshipSystems},
        sections::prelude::*,
    };
}
