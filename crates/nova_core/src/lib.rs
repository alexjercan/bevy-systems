//! TODO: Add description in this crate

use std::time::Duration;

use bevy::{
    app::ScheduleRunnerPlugin,
    log::{Level, LogPlugin},
    prelude::*,
    window::PresentMode,
    winit::WinitPlugin,
};

use bevy_common_systems::prelude::*;
use nova_assets::prelude::*;
use nova_gameplay::prelude::*;

#[cfg(feature = "debug")]
use nova_debug::DebugPlugin;

pub mod prelude {
    pub use super::{new_gui_app, new_headless_app, GameStates};
    pub use bevy_common_systems::prelude::*;
    pub use nova_assets::prelude::*;
    pub use nova_gameplay::prelude::*;

    #[cfg(feature = "debug")]
    pub use nova_debug::prelude::*;
}

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
    app.add_plugins(GameAssetsPlugin);
    app.add_plugins(GameplayPlugin { render: true });

    #[cfg(feature = "debug")]
    app.add_plugins(DebugPlugin);

    // When we enter the Loaded state, switch to Playing state
    // TODO: Here we will add a MainMenu state before Playing
    app.add_systems(
        OnEnter(GameAssetsStates::Loaded),
        |mut state: ResMut<NextState<GameStates>>| {
            state.set(GameStates::Playing);
        },
    );

    // Setup the status UI when entering the Playing state
    app.add_systems(OnEnter(GameStates::Playing), setup_status_ui);

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

fn setup_status_ui(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        DespawnOnExit(GameStates::Playing),
        status_bar(StatusBarRootConfig::default()),
    ));

    commands.spawn((
        DespawnOnExit(GameStates::Playing),
        status_bar_item(StatusBarItemConfig {
            icon: Some(game_assets.fps_icon.clone()),
            value_fn: status_fps_value_fn(),
            color_fn: status_fps_color_fn(),
            prefix: "".to_string(),
            suffix: "fps".to_string(),
        }),
    ));
    commands.spawn((
        DespawnOnExit(GameStates::Playing),
        status_bar_item(StatusBarItemConfig {
            icon: None,
            value_fn: status_version_value_fn(env!("CARGO_PKG_VERSION").to_string()),
            color_fn: status_version_color_fn(),
            prefix: "v".to_string(),
            suffix: "".to_string(),
        }),
    ));
}
