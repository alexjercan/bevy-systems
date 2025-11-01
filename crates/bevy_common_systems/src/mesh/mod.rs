//! Mesh utilities for Bevy games.

pub mod explode;
pub mod slicer;

pub mod prelude {
    pub use super::{explode::prelude::*, slicer::mesh_slice};
}
