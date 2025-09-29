mod setup;
mod camera;

pub mod prelude {
    pub use crate::setup::{new_gui_app, new_headless_app};
    pub use crate::camera::prelude::*;
}
