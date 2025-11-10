//! Mesh utilities for Bevy games.

pub mod explode;
pub mod noise;
pub mod slicer;
pub mod sphere;
pub mod util;

pub mod prelude {
    pub use super::{
        explode::prelude::*, noise::apply_noise_to_mesh, slicer::mesh_slice,
        sphere::octahedron_sphere, util::TriangleMeshBuilder,
    };
}
