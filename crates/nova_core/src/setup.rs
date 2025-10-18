//! This module contains functions to create new Bevy apps with different configurations.

use std::time::Duration;

use bevy::{
    app::ScheduleRunnerPlugin,
    log::{Level, LogPlugin},
    prelude::*,
    window::PresentMode,
    winit::WinitPlugin,
};

#[cfg(feature = "debug")]
use nova_debug::DebugPlugin;

/// Game states for the application.
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameStates {
    #[default]
    Loading,
    Playing,
}

pub fn new_gui_app() -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .set(assets_plugin())
            .set(log_plugin())
            .set(window_plugin()),
    );
    // Experimental Plugins
    app.add_plugins(bevy::ui_widgets::UiWidgetsPlugins);

    app.init_state::<GameStates>();

    app.add_plugins(bevy_enhanced_input::EnhancedInputPlugin);
    app.add_plugins(nova_assets::prelude::GameAssetsPlugin);
    app.add_plugins(nova_gameplay::prelude::GameplayPlugin { render: true });

    #[cfg(feature = "debug")]
    app.add_plugins(DebugPlugin);

    app.add_systems(
        OnEnter(nova_assets::prelude::GameAssetsStates::Loaded),
        |mut state: ResMut<NextState<GameStates>>| {
            state.set(GameStates::Playing);
        },
    );

    app
}

pub fn new_headless_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .build()
            .set(AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(log_plugin())
            .disable::<WinitPlugin>(),
        ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 64.0)),
    ));

    app
}

fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: format!("NovaProtocol - {}", env!("CARGO_PKG_VERSION")),
            resolution: (1024, 768).into(),
            present_mode: PresentMode::AutoVsync,
            // Bind to canvas included in `index.html`
            canvas: Some("#bevy".to_owned()),
            fit_canvas_to_parent: true,
            // set to true if we want to capture tab etc in wasm
            prevent_default_event_handling: true,
            ..Default::default()
        }),
        ..default()
    }
}

fn log_plugin() -> LogPlugin {
    LogPlugin {
        level: Level::INFO,
        filter: log_filter_str().to_string(),
        ..default()
    }
}

fn log_filter_str<'a>() -> &'a str {
    if cfg!(feature = "debug") {
        "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn"
    } else {
        "wgpu=error,bevy_render=warn,bevy_ecs=warn,bevy_time=warn,naga=warn"
    }
}

fn assets_plugin() -> AssetPlugin {
    AssetPlugin {
        meta_check: bevy::asset::AssetMetaCheck::Never,
        ..default()
    }
}
