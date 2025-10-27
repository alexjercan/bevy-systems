//! The simulation plugin. This plugin should contain all the gameplay related logic.

use bevy::prelude::*;
use nova_assets::prelude::*;
use nova_gameplay::prelude::*;

pub mod prelude {}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationPluginSet;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(super::GameStates::Simulation),
            (setup_camera_controller,),
        );
    }
}

fn setup_camera_controller(mut commands: Commands, game_assets: Res<GameAssets>) {
    // Spawn a 3D camera with a chase camera component
    // commands.spawn((
    //     DespawnOnExit(super::GameStates::Simulation),
    //     Name::new("Chase Camera"),
    //     Camera3d::default(),
    //     ChaseCamera::default(),
    //     SpaceshipCameraControllerMarker,
    //     Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     SkyboxConfig {
    //         cubemap: game_assets.cubemap.clone(),
    //         brightness: 1000.0,
    //     },
    // ));
}
