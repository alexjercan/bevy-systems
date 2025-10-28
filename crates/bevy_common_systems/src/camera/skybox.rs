//! A Plugin that adds a skybox to a Bevy application.
//!
//! The skybox should be an image in a cubemap format. Basically you want to have 6 square images
//! stacked vertically in a single image file. This plugin will reinterpret the image as a cubemap
//! and set it up to be used as a skybox for the camera entity that has the `SkyboxConfig`
//! component.

use bevy::{
    core_pipeline::Skybox,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};

pub mod prelude {
    pub use super::SkyboxConfig;
    pub use super::SkyboxPlugin;
}

/// Component that should be added on the camera you want to have the Skybox
#[derive(Component, Clone, Debug)]
#[require(Camera)]
pub struct SkyboxConfig {
    pub cubemap: Handle<Image>,
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

/// A Plugin for the skybox that will set up the image and integrate it with the camera
pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        debug!("SkyboxPlugin: build");

        app.add_observer(setup_skybox_camera);
    }
}

fn setup_skybox_camera(
    insert: On<Insert, SkyboxConfig>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    q_config: Query<&SkyboxConfig, With<Camera>>,
) {
    let entity = insert.entity;
    trace!("setup_skybox_camera: entity {:?}", entity);

    let Ok(config) = q_config.get(entity) else {
        warn!("SkyboxCubemap component must be added to a Camera entity");
        return;
    };

    let image = images.get_mut(&config.cubemap).unwrap();
    if image.texture_descriptor.array_layer_count() == 1 {
        image.reinterpret_stacked_2d_as_array(image.height() / image.width());
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
    }

    commands.entity(insert.entity).insert((Skybox {
        image: config.cubemap.clone(),
        brightness: config.brightness,
        ..default()
    },));
}
