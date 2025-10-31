//! Gameplay related functionality for Nova Protocol.
//!
//! Nova Protocol specific systems and components.

pub mod damage;
pub mod destruction;
pub mod spaceship;
pub use bevy_common_systems;

pub mod prelude {
    // Re-export bevy_common_systems prelude
    pub use bevy_common_systems::prelude::*;

    pub use super::{damage::prelude::*, destruction::prelude::*, spaceship::prelude::*};
}
