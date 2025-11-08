//! Mesh utilities for Bevy games.

pub mod explode;
pub mod slicer;
pub mod sphere;
pub mod noise;
pub mod util;

pub mod prelude {
    pub use super::{
        explode::prelude::*, slicer::mesh_slice, sphere::octahedron_sphere, util::TriangleMeshBuilder,
        noise::apply_noise_to_mesh,
    };
}
