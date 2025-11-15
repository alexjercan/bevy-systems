//! Mesh utilities for Bevy games.
//!
//! This module provides tools for procedural mesh generation and mesh manipulation,
//! including building custom triangle meshes and exploding meshes into fragments for
//! visual or gameplay effects.

pub mod builder;
pub mod explode;

/// The prelude re-exports the most commonly used mesh utilities.
///
/// Use `bevy_common_systems::mesh::prelude::*` to easily access `TriangleMeshBuilder` for mesh
/// construction and the `explode` module for mesh explosion features.
pub mod prelude {
    pub use super::{builder::TriangleMeshBuilder, explode::prelude::*};
}
