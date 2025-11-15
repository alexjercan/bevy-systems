//! A plugin that applies default post processing settings to 3D cameras.
//!
//! This plugin automatically enables tonemapping and bloom on any entity
//! that receives a `Camera3d` component. It is meant to provide a simple
//! and visually appealing default look without requiring any manual setup.
//!
//! The current defaults are:
//! - Tonemapping::TonyMcMapface
//! - Bloom::NATURAL
//!
//! Example usage:
//!
//! ```rust
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugin(PostProcessingDefaultPlugin);
//!
//! // Any spawned Camera3d will automatically receive bloom and tonemapping:
//! commands.spawn(Camera3d::default());
//! ```
//!
//! If you want different defaults or more control over post processing,
//! consider writing your own plugin or inserting the components manually.

use bevy::{core_pipeline::tonemapping::Tonemapping, post_process::bloom::Bloom, prelude::*};

pub mod prelude {
    pub use super::PostProcessingDefaultPlugin;
}

/// A plugin that applies default post processing settings.
///
/// When a `Camera3d` is added to an entity, this plugin automatically inserts
/// `Tonemapping::TonyMcMapface` and `Bloom::NATURAL`.
pub struct PostProcessingDefaultPlugin;

impl Plugin for PostProcessingDefaultPlugin {
    fn build(&self, app: &mut App) {
        debug!("PostProcessingDefaultPlugin: build");

        app.add_observer(setup_post_processing_camera);
    }
}

fn setup_post_processing_camera(insert: On<Insert, Camera3d>, mut commands: Commands) {
    let entity = insert.entity;
    trace!("setup_post_processing_camera: entity {:?}", entity);

    commands
        .entity(entity)
        .insert((Tonemapping::TonyMcMapface, Bloom::NATURAL));
}
