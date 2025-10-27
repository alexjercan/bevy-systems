//! Gameplay related functionality for Nova Protocol.

pub mod damage;
pub mod destruction;
pub mod spaceship;

pub mod prelude {
    pub use super::damage::prelude::*;
    pub use super::destruction::prelude::*;
    pub use super::spaceship::prelude::*;
}
