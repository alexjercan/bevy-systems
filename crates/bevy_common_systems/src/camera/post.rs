use bevy::{core_pipeline::tonemapping::Tonemapping, post_process::bloom::Bloom, prelude::*};

pub mod prelude {
    pub use super::PostProcessingDefaultPlugin;
}

/// A Plugin for the post processing with some default settings
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
