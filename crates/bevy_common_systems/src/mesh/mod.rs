//! Mesh utilities for Bevy games.

pub mod explode;
pub mod builder;

pub mod prelude {
    pub use super::{explode::prelude::*, builder::TriangleMeshBuilder};
}
