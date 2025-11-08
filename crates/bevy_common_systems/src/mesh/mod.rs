//! Mesh utilities for Bevy games.

pub mod explode;
pub mod slicer;
pub mod util;
pub mod sphere;

pub mod prelude {
    pub use super::{explode::prelude::*, slicer::mesh_slice, sphere::octahedron_sphere};
}
