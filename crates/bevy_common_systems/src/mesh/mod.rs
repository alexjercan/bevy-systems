//! Mesh utilities for Bevy games.

pub mod explode;
pub mod util;

pub mod prelude {
    pub use super::{explode::prelude::*, util::TriangleMeshBuilder};
}
