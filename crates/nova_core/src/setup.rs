//! This module contains functions to create new Bevy apps with different configurations.

use std::time::Duration;

use avian3d::prelude::*;
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

    app.init_state::<GameStates>();

    // Experimental Plugins
    app.add_plugins(bevy::ui_widgets::UiWidgetsPlugins);

    // Enhanced input handling
    app.add_plugins(bevy_enhanced_input::EnhancedInputPlugin);

    // Game Assets Plugin
    app.add_plugins(nova_assets::prelude::GameAssetsPlugin);

    // Bevy Common Systems - WASD Camera
    app.add_plugins(bevy_common_systems::prelude::WASDCameraPlugin);
    app.add_plugins(bevy_common_systems::prelude::WASDCameraControllerPlugin);
    // Bevy Common Systems - Rendering
    app.add_plugins(bevy_common_systems::prelude::SkyboxPlugin);
    app.add_plugins(bevy_common_systems::prelude::PostProcessingDefaultPlugin);
    // Chase Camera Plugin to have a 3rd person camera following the spaceship
    app.add_plugins(bevy_common_systems::prelude::ChaseCameraPlugin);
    // Point Rotation Plugin to convert mouse movement to a target rotation
    app.add_plugins(bevy_common_systems::prelude::PointRotationPlugin);
    // for debug to have a random orbiting object
    app.add_plugins(bevy_common_systems::prelude::SphereRandomOrbitPlugin);
    // Rotation Plugin for the turret facing direction
    app.add_plugins(bevy_common_systems::prelude::SmoothLookRotationPlugin);
    // Other helper plugins
    app.add_plugins(bevy_common_systems::prelude::TempEntityPlugin);
    app.add_plugins(bevy_common_systems::prelude::ProjectilePlugin { render: true });

    // We need to enable the physics plugins to have access to RigidBody and other components.
    // We will also disable gravity for this example, since we are in space, duh.
    app.add_plugins(PhysicsPlugins::default().set(PhysicsInterpolationPlugin::interpolate_all()));
    app.add_plugins(PhysicsPickingPlugin);
    app.insert_resource(Gravity::ZERO);

    // GamePlay Plugins
    app.add_plugins(nova_gameplay::prelude::SpaceshipPlugin { render: true });

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
