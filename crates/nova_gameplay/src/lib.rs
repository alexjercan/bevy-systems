//! Gameplay related functionality for Nova Protocol.
//!
//! Nova Protocol specific systems and components.

pub mod damage;
pub mod destruction;
pub mod spaceship;
pub use bevy_common_systems;

pub mod prelude {
    pub use super::damage::prelude::*;
    pub use super::destruction::prelude::*;
    pub use super::spaceship::prelude::*;

    // Re-export bevy_common_systems prelude
    pub use bevy_common_systems::prelude::*;
}
