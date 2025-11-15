pub mod inspector;
pub mod wireframe;

pub mod prelude {
    pub use super::{inspector::InpsectorDebugPlugin, wireframe::WireframeDebugPlugin};
}
