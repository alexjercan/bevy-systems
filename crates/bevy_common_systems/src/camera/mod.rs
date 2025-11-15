//! Camera related modules.
//!
//! This module contains several camera systems used in the game:
//! - Chase camera
//! - Post processing camera utilities
//! - Skybox rendering helpers
//! - WASD style free camera
//!
//! You can import the commonly used types and plugins through the prelude:
//!
//! ```rust
//! use bevy_common_systems::camera::prelude::*;
//! ```

mod chase;
mod post;
mod skybox;
mod wasd;

/// Re-exports commonly used camera systems and utilities for convenience.
///
/// ```rust
/// use bevy_common_systems::camera::prelude::*;
/// ```
pub mod prelude {
    pub use super::{chase::prelude::*, post::prelude::*, skybox::prelude::*, wasd::prelude::*};
}
