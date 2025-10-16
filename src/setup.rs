//! This module contains functions to create new Bevy apps with different configurations.

use std::time::Duration;

use bevy::{
    app::ScheduleRunnerPlugin,
    log::{Level, LogPlugin},
    prelude::*,
    window::PresentMode,
    winit::WinitPlugin,
};

fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: format!("SpaceGame - {}", env!("CARGO_PKG_VERSION")),
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
        "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn,bevy_systems=debug"
    } else {
        "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn,bevy_systems=info"
    }
}

pub fn new_gui_app() -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .set(AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(log_plugin())
            .set(window_plugin()),
    );

    if cfg!(feature = "debug") {
        app.add_plugins(debug::InpsectorDebugPlugin);
    }

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

mod debug {
    use bevy::prelude::*;
    use bevy_inspector_egui::{
        bevy_egui::{EguiContext, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext},
        bevy_inspector, egui, DefaultInspectorConfigPlugin,
    };

    pub struct InpsectorDebugPlugin;

    impl Plugin for InpsectorDebugPlugin {
        fn build(&self, app: &mut App) {
            app
                // Bevy egui inspector
                .add_plugins(EguiPlugin::default())
                .add_plugins(DefaultInspectorConfigPlugin)
                .add_systems(EguiPrimaryContextPass, inspector_ui)
                .add_systems(Startup, setup);
        }
    }

    fn setup(mut _commands: Commands) {
        // commands.spawn((
        //     Camera2d,
        //     Camera {
        //         order: 2,
        //         ..default()
        //     },
        //     Name::new("Debug Camera"),
        //     RenderLayers::layer(2),
        // ));
    }

    fn inspector_ui(world: &mut World) {
        let mut egui_context = world
            .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
            .single(world)
            .expect("EguiContext not found")
            .clone();

        egui::Window::new("UI").show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                bevy_inspector::ui_for_world(world, ui);
            });
        });
    }
}
