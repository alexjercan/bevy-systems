//! Mesh utilities for Bevy games.

pub mod slicer;
pub mod explode;

pub mod prelude {
    pub use super::slicer::mesh_slice;
    pub use super::explode::prelude::*;
}
