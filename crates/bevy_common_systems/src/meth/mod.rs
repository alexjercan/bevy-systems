//! Utilities for vector math and interpolation in 3D space.
//!
//! This module provides helper functions and traits for smooth value interpolation
//! (`LerpSnap`) and spherical coordinate conversions and operations (`sphere`).

pub mod lerp;
pub mod sphere;

/// The prelude re-exports the most commonly used math utilities.
///
/// Use `bevy_common_systems::meth::prelude::*` to easily access `LerpSnap` for smooth
/// interpolation and all spherical math functions from `sphere`.
pub mod prelude {
    pub use super::{lerp::LerpSnap, sphere::*};
}
