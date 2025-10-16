//! TODO: Add description in this crate

pub mod setup;

pub mod prelude {
    pub use crate::setup::{GameStates, new_gui_app, new_headless_app};
    pub use bevy_common_systems::prelude::*;
    pub use nova_assets::prelude::*;
    pub use nova_gameplay::prelude::*;
}
