//! TODO: Add description in this crate

pub mod setup;

pub use nova_gameplay;

pub mod prelude {
    pub use crate::setup::{new_gui_app, new_headless_app, GameStates};
    pub use bevy_common_systems::prelude::*;
    pub use nova_assets::prelude::*;
    pub use nova_gameplay::prelude::*;

    #[cfg(feature = "debug")]
    pub use nova_debug::prelude::*;
}
