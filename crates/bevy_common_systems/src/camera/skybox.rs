//! A plugin that adds a skybox to a Bevy application.
//!
//! This plugin takes a single stacked cubemap image, converts it into a cubemap
//! texture, and attaches it to any camera that has a `SkyboxConfig` component.
//!
//! The cubemap image must contain 6 square images stacked vertically in a
//! single file. The layout should look like this:
//!
//!     +----------------+
//!     |     face 0     |
//!     +----------------+
//!     |     face 1     |
//!     +----------------+
//!     |     face 2     |
//!     +----------------+
//!     |     face 3     |
//!     +----------------+
//!     |     face 4     |
//!     +----------------+
//!     |     face 5     |
//!     +----------------+
//!
//! The image must have a height that is exactly 6 times its width. The plugin
//! will reinterpret this stacked image as a 6 layer 2D texture array and then
//! convert it to a proper cubemap texture view, which can be used by Bevy's
//! built in `Skybox` component.
//!
//! To use the skybox:
//!
//! ```rust
//! commands.spawn((
//!     Camera3d::default(),
//!     SkyboxConfig {
//!         cubemap: asset_server.load("skybox.png"),
//!         brightness: 1000.0,
//!     }
//! ));
//! ```
//!
//! The plugin will automatically configure the skybox when the component
//! is inserted.

use bevy::{
    core_pipeline::Skybox,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};

pub mod prelude {
    pub use super::{SkyboxConfig, SkyboxPlugin};
}

/// Component used to enable a skybox on a specific camera.
///
/// Add this component to any camera entity. The plugin will then
/// reinterpret the provided cubemap image as a cubemap texture and
/// attach a Bevy `Skybox` component to the same entity.
#[derive(Component, Clone, Debug)]
#[require(Camera)]
pub struct SkyboxConfig {
    /// Handle to the stacked cubemap image.
    pub cubemap: Handle<Image>,

    /// Skybox brightness multiplier.
    pub brightness: f32,
}

impl Default for SkyboxConfig {
    fn default() -> Self {
        Self {
            cubemap: Handle::default(),
            brightness: 1000.0,
        }
    }
}

/// Plugin that enables cubemap skyboxes.
///
/// When a `SkyboxConfig` is added to a camera, the plugin will:
/// - reinterpret the provided stacked image as a cubemap
/// - configure the texture view to be a cube
/// - attach a `Skybox` component to the camera
pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        debug!("SkyboxPlugin: build");

        app.add_observer(setup_skybox_camera);
    }
}

/// Handle setup when a SkyboxConfig is inserted on a camera.
fn setup_skybox_camera(
    insert: On<Insert, SkyboxConfig>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    q_config: Query<&SkyboxConfig, With<Camera>>,
) {
    let entity = insert.entity;
    trace!("setup_skybox_camera: entity {:?}", entity);

    let Ok(config) = q_config.get(entity) else {
        error!(
            "setup_skybox_camera: entity {:?} not found in q_config",
            entity
        );
        return;
    };

    let image = images.get_mut(&config.cubemap).unwrap();

    // Only reinterpret if it is not already an array texture.
    if image.texture_descriptor.array_layer_count() == 1 {
        // Convert stacked image into a 6 layer array.
        image.reinterpret_stacked_2d_as_array(image.height() / image.width());

        // Mark the texture view as a cubemap so Bevy can use it as a skybox.
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
    }

    commands.entity(entity).insert((Skybox {
        image: config.cubemap.clone(),
        brightness: config.brightness,
        ..default()
    },));
}
