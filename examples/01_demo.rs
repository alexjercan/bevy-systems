mod helpers;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use clap::Parser;
use helpers::*;
use nova_protocol::prelude::*;

#[derive(Parser)]
#[command(name = "spaceship_demo_01")]
#[command(version = "0.1")]
#[command(about = "Demo for the first version", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = new_gui_app();
    app.add_plugins(EnhancedInputPlugin);

    app.add_plugins(GameAssetsPlugin);
    app.add_plugins(GameSkyboxPlugin);
    app.add_plugins(WASDCameraControllerPlugin);

    app.add_systems(
        OnEnter(GameStates::Playing),
        (setup_scene, setup_simple_scene),
    );

    app.run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Name::new("WASD Camera"),
        WASDCameraController,
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
