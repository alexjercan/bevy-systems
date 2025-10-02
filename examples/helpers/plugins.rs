use super::systems::*;
use bevy::{
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};
use bevy_asset_loader::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameStates {
    #[default]
    Loading,
    Playing,
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        // Setup the asset loader to load assets during the loading state.
        app.init_state::<GameStates>();
        app.add_loading_state(
            LoadingState::new(GameStates::Loading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>(),
        );

        app.add_systems(OnEnter(GameStates::Playing), setup);
    }
}

pub struct DebugGizmosPlugin;

impl Plugin for DebugGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_debug_gizmos_axis);
    }
}

#[derive(AssetCollection, Resource, Clone)]
pub struct GameAssets {
    #[asset(path = "textures/cubemap.png")]
    pub cubemap: Handle<Image>,
}

fn setup(game_assets: Res<GameAssets>, mut images: ResMut<Assets<Image>>) {
    let image = images.get_mut(&game_assets.cubemap).unwrap();
    if image.texture_descriptor.array_layer_count() == 1 {
        image.reinterpret_stacked_2d_as_array(image.height() / image.width());
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
    }
}
