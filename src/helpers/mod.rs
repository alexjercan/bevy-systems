#![allow(unused_imports)]

#[cfg(feature = "debug")]
pub mod debug;

pub mod camera;
pub mod chunk_map;
pub mod meth;
pub mod tilemap;

pub mod prelude {
    #[cfg(feature = "debug")]
    pub use super::debug::prelude::*;

    pub use super::camera::prelude::*;
    pub use super::chunk_map::prelude::*;
    pub use super::meth::prelude::*;
    pub use super::tilemap::prelude::*;
}
