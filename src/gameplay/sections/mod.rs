//! This module contains all the sections of a spaceship.

// mod cargo_section;
mod engine_section;
mod hull_section;
mod root_section;
mod turret_section;
mod crew_section;

pub mod prelude {
    // pub use super::cargo_section::prelude::*;
    pub use super::engine_section::prelude::*;
    pub use super::hull_section::prelude::*;
    pub use super::root_section::prelude::*;
    pub use super::turret_section::prelude::*;
    pub use super::crew_section::prelude::*;
}
