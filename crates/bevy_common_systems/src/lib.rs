//! Common Gameplay Components and Systems for Bevy Games.
//!
//! Fully copy-pastable crate for common gameplay components and systems.

pub mod camera;
pub mod health;
pub mod helpers;
pub mod mesh;
pub mod meth;
pub mod modding;
pub mod physics;
pub mod transform;
pub mod ui;
pub use bevy_common_systems_macros;

pub mod prelude {
    pub use bevy_common_systems_macros::*;

    pub use crate::{
        camera::prelude::*, health::prelude::*, helpers::prelude::*, mesh::prelude::*,
        meth::prelude::*, modding, modding::prelude::*, physics::prelude::*, transform::prelude::*,
        ui::prelude::*,
    };
}
