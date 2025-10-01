use bevy::{
    core_pipeline::Skybox,
    prelude::*,
    render::{render_resource::{TextureViewDescriptor, TextureViewDimension}, view::RenderLayers},
};
use bevy_asset_loader::prelude::*;
use super::systems::*;
use super::setup_systems::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameStates {
    #[default]
    Loading,
    Playing,
}

pub struct PrettyScenePlugin;

impl Plugin for PrettyScenePlugin {
    fn build(&self, app: &mut App) {
        // Setup the asset loader to load assets during the loading state.
        app.init_state::<GameStates>();
        app.add_loading_state(
            LoadingState::new(GameStates::Loading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>(),
        );

        app.add_systems(OnEnter(GameStates::Playing), (setup, setup_simple_scene));
        app.add_systems(Update, follow_main_camera);
        app.add_systems(Update, draw_debug_gizmos_axis);
    }
}

#[derive(AssetCollection, Resource, Clone)]
pub struct GameAssets {
    #[asset(path = "textures/cubemap.png")]
    pub cubemap: Handle<Image>,
}

fn setup(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut images: ResMut<Assets<Image>>,
) {
    let image = images.get_mut(&game_assets.cubemap).unwrap();
    if image.texture_descriptor.array_layer_count() == 1 {
        image.reinterpret_stacked_2d_as_array(image.height() / image.width());
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
    }

    commands.spawn((
        Name::new("Skybox Camera"),
        Camera3d::default(),
        Camera {
            order: -1,
            ..default()
        },
        Transform::default(),
        GlobalTransform::default(),
        Skybox {
            image: game_assets.cubemap.clone(),
            brightness: 1000.0,
            ..default()
        },
        RenderLayers::layer(1),
    ));
}

fn follow_main_camera(
    camera: Single<&Transform, (With<Camera3d>, Without<Skybox>)>,
    skybox_camera: Single<&mut Transform, (With<Camera3d>, With<Skybox>)>,
) {
    let transform = camera.into_inner();
    let mut skybox_transform = skybox_camera.into_inner();
    skybox_transform.translation = transform.translation;
    skybox_transform.rotation = transform.rotation;
}
