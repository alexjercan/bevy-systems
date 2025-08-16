use bevy::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameStates {
    #[default]
    AssetLoading,
    Playing,
}
