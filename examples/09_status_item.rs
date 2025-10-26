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
    let mut app = new_gui_app();

    app.add_systems(
        OnEnter(GameStates::Playing),
        (setup_camera, setup_status_ui),
    );

    app.run();
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
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(status_bar(StatusBarRootConfig::default()));
    commands.spawn(status_bar_item(StatusBarItemConfig {
        icon: Some(game_assets.fps_icon.clone()),
        value_fn: status_fps_value_fn(),
        color_fn: status_fps_color_fn(),
        prefix: "".to_string(),
        suffix: "fps".to_string(),
    }));
    commands.spawn(status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: status_version_value_fn(env!("CARGO_PKG_VERSION").to_string()),
        color_fn: status_version_color_fn(),
        prefix: "v".to_string(),
        suffix: "".to_string(),
    }));
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
