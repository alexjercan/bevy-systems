use std::process::Command;
use std::sync::Arc;

use bevy::prelude::*;
use clap::Parser;
use nova_protocol::prelude::*;

#[derive(Parser)]
#[command(name = "09_status")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how to use the status bar nova_protocol", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = AppBuilder::new().with_game_plugins(custom_plugin).build();

    app.run();
}

fn custom_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameStates::Simulation), setup_camera);
    app.add_observer(setup_status_ui);
}

fn setup_camera(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        Name::new("Main Camera"),
        Camera3d::default(),
        WASDCameraController,
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        SkyboxConfig {
            cubemap: game_assets.cubemap.clone(),
            brightness: 1000.0,
        },
    ));
}

fn setup_status_ui(
    _: On<Add, StatusBarRootMarker>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(status_bar_item(StatusBarItemConfig {
        icon: Some(asset_server.load("icons/linux.png")),
        value_fn: |_| {
            let output = Command::new("uname")
                .arg("-r")
                .output()
                .expect("Failed to execute uname");

            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Some(Arc::new(result) as Arc<dyn StatusValue>)
        },
        color_fn: |_| Some(Color::srgb(1.0, 1.0, 1.0)),
        prefix: "kernel".to_string(),
        suffix: "".to_string(),
    }));
}
