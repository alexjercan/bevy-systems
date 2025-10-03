mod setup;
mod camera;
mod physics;
mod transform;
mod meth;
mod gameplay;

pub mod prelude {
    pub use crate::setup::{new_gui_app, new_headless_app};
    pub use crate::camera::prelude::*;
    pub use crate::physics::prelude::*;
    pub use crate::transform::prelude::*;
    pub use crate::meth::prelude::*;
    pub use crate::gameplay::prelude::*;
}
