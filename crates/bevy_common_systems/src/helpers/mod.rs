//! Utilities and controllers for Bevy.
//!
//! This module provides a collection of helper systems and components, including:
//! - [`despawn`] - utilities to safely despawn entities when a marker component is added.
//! - [`temp`] - temporary helpers for testing entity lifetimes.
//! - [`wasd`] - WASD-style camera movement and mouse look controllers.
//!
//! The `prelude` module re-exports the most commonly used types from all submodules for convenience.
//! This allows you to import everything you need with a single line:
//!
//! ```rust
//! use bevy_common_systems::heleprs::prelude::*;
//! ```

pub mod despawn;
pub mod temp;
pub mod wasd;

/// Prelude module re-exporting the most commonly used types from all submodules.
pub mod prelude {
    pub use super::{despawn::prelude::*, temp::prelude::*, wasd::prelude::*};
}
