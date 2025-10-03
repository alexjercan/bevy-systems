mod setup;
mod camera;
mod transform;
mod gameplay;

pub mod prelude {
    pub use crate::setup::{new_gui_app, new_headless_app};
    pub use crate::camera::prelude::*;
    pub use crate::transform::prelude::*;
    pub use crate::gameplay::prelude::*;
}
