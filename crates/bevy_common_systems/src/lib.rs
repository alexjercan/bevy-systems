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

pub mod prelude {
    pub use crate::camera::prelude::*;
    pub use crate::health::prelude::*;
    pub use crate::helpers::prelude::*;
    pub use crate::mesh::prelude::*;
    pub use crate::meth::prelude::*;
    pub use crate::modding::prelude::*;
    pub use crate::physics::prelude::*;
    pub use crate::transform::prelude::*;
    pub use crate::ui::prelude::*;

    pub use crate::modding;
    pub use bevy_common_systems_macros::*;
}
