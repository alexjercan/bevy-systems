//! Top-level game events module.

pub mod events;

/// Re-export commonly used items from the `events` module.
pub mod prelude {
    pub use super::events::prelude::*;
}
