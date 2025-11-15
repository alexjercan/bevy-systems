//! Debug utilities for your Bevy game.
//!
//! This module groups optional debugging tools such as:
//! - **WireframeDebugPlugin** - toggles global wireframe rendering.
//! - **InspectorDebugPlugin** - enables the Bevy inspector (if enabled).
//!
//! ## Usage
//! Add whichever plugins you want, or pull them all via the `prelude`:
//!
//! ```rust
//! use bevy_common_systems::debug::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(WireframeDebugPlugin)
//!         .add_plugins(InspectorDebugPlugin)
//!         .run();
//! }
//! ```
//!
//! The `prelude` module re-exports the most commonly used debug plugins.

pub mod inspector;
pub mod wireframe;

/// Re-exports commonly used debug plugins for convenience.
///
/// ```rust
/// use bevy_common_systems::debug::prelude::*;
/// ```
pub mod prelude {
    pub use super::{inspector::InspectorDebugPlugin, wireframe::WireframeDebugPlugin};
}
