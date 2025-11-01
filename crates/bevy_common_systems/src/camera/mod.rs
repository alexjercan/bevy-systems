//! Camera related modules

mod chase;
mod post;
mod skybox;
mod wasd;

pub mod prelude {
    pub use super::{chase::prelude::*, post::prelude::*, skybox::prelude::*, wasd::prelude::*};
}
