//! Mesh utilities for Bevy games.

pub mod builder;
pub mod explode;

pub mod prelude {
    pub use super::{builder::TriangleMeshBuilder, explode::prelude::*};
}
