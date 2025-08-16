//! TODO: documentation for camera module

pub mod rts_camera;
pub mod wasd_camera;

pub mod prelude {
    pub use super::rts_camera::*;
    pub use super::wasd_camera::*;
}
