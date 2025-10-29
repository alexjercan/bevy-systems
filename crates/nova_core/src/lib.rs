//! TODO: Add description in this crate

use avian3d::prelude::*;

use bevy::{
    app::Plugins,
    log::{Level, LogPlugin},
    prelude::*,
    window::PresentMode,
};

use nova_assets::prelude::*;
use nova_gameplay::{bevy_common_systems, prelude::*};

#[cfg(feature = "debug")]
use nova_debug::DebugPlugin;

pub mod editor;
pub mod simulation;
pub use nova_gameplay;

pub mod prelude {
    pub use super::{AppBuilder, GameStates};

    // NOTE: These are temporary, until I finis the refactor to move everything to new_gui_app
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
    Simulation,
    Editor,
}

pub struct AppBuilder {
    app: App,
    use_default_plugins: bool,
    render: bool,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            app: App::new(),
            use_default_plugins: true,
            render: true,
        }
    }

    pub fn with_game_plugins<M>(mut self, plugins: impl Plugins<M>) -> Self {
        self.app.add_plugins(plugins);
        self.use_default_plugins = false;
        self
    }

    pub fn with_rendering(mut self, render: bool) -> Self {
        self.render = render;
        self
    }

    pub fn build(mut self) -> App {
        self.app.add_plugins(
            DefaultPlugins
                .build()
                .set(assets_plugin())
                .set(log_plugin())
                .set(window_plugin()),
        );
        // Experimental Plugins
        self.app.add_plugins(bevy::ui_widgets::UiWidgetsPlugins);

        self.app.init_state::<GameStates>();

        self.app
            .add_plugins(bevy_enhanced_input::EnhancedInputPlugin);
        self.app.add_plugins(GameAssetsPlugin);
        self.app.add_plugins(CorePlugin {
            render: self.render,
        });

        // Add default game plugins if none were provided
        if self.use_default_plugins {
            self.app.add_plugins(simulation::SimulationPlugin);
            self.app.add_plugins(editor::EditorPlugin);
        }

        #[cfg(feature = "debug")]
        self.app.add_plugins(DebugPlugin);

        // When we enter the Loaded state, switch to Playing state
        // Setup the status UI when entering the Playing state
        // TODO: Here we will add a MainMenu state before Playing
        self.app.add_systems(
            OnEnter(GameAssetsStates::Loaded),
            (
                |mut state: ResMut<NextState<GameStates>>| {
                    state.set(GameStates::Simulation);
                },
                setup_status_ui,
            ),
        );

        self.app
    }
}

// pub fn new_headless_app() -> App {
//     let mut app = App::new();
//     app.add_plugins((
//         DefaultPlugins
//             .build()
//             .set(AssetPlugin {
//                 meta_check: bevy::asset::AssetMetaCheck::Never,
//                 ..default()
//             })
//             .set(log_plugin())
//             .disable::<WinitPlugin>(),
//         ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 64.0)),
//     ));
//
//     app
// }

#[derive(Default, Clone, Debug)]
struct CorePlugin {
    pub render: bool,
}

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        // We need to enable the physics plugins to have access to RigidBody and other components.
        // We will also disable gravity for this example, since we are in space, duh.
        app.add_plugins(PhysicsPlugins::default().with_collision_hooks::<TurretProjectileHooks>());
        app.add_plugins(PhysicsPickingPlugin);
        app.insert_resource(Gravity::ZERO);

        // FIXME: For now we disable particle effects on wasm because it's not working
        #[cfg(not(target_family = "wasm"))]
        app.add_plugins(bevy_hanabi::HanabiPlugin);

        // Bevy Common Systems - WASD Camera
        app.add_plugins(bevy_common_systems::prelude::WASDCameraPlugin);
        app.add_plugins(bevy_common_systems::prelude::WASDCameraControllerPlugin);
        // Chase Camera Plugin to have a 3rd person camera following the spaceship
        app.add_plugins(bevy_common_systems::prelude::ChaseCameraPlugin);
        // Bevy Common Systems - Rendering
        app.add_plugins(bevy_common_systems::prelude::SkyboxPlugin);
        app.add_plugins(bevy_common_systems::prelude::PostProcessingDefaultPlugin);
        // Point Rotation Plugin to convert linear movement to a target rotation
        app.add_plugins(bevy_common_systems::prelude::PointRotationPlugin);
        // for debug to have a random orbiting object
        app.add_plugins(bevy_common_systems::prelude::SphereRandomOrbitPlugin);
        // Rotation Plugin for the turret facing direction
        app.add_plugins(bevy_common_systems::prelude::SmoothLookRotationPlugin);
        // Sphere Orbit Plugin
        app.add_plugins(bevy_common_systems::prelude::SphereOrbitPlugin);
        app.add_plugins(bevy_common_systems::prelude::DirectionalSphereOrbitPlugin);
        // Other helper plugins
        app.add_plugins(bevy_common_systems::prelude::TempEntityPlugin);
        // Core Mechanics
        app.add_plugins(bevy_common_systems::prelude::CollisionDamagePlugin);
        app.add_plugins(bevy_common_systems::prelude::HealthPlugin);

        // UI Plugins
        app.add_plugins(nova_gameplay::bevy_common_systems::prelude::StatusBarPlugin);

        // Core Plugins for simulation
        app.add_plugins(nova_gameplay::spaceship::SpaceshipPlugin {
            render: self.render,
        });
        app.add_plugins(nova_gameplay::damage::DamagePlugin);
        app.add_plugins(nova_gameplay::destruction::DestructionHealthPlugin);

        // Diagnostics
        if !app.is_plugin_added::<bevy::diagnostic::FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
        }

        // Configure system Sets
        app.configure_sets(
            Update,
            SpaceshipSystems::Input.run_if(in_state(GameStates::Simulation)),
        );
    }
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
        "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn,bevy_common_systems=debug,nova_gameplay=debug,nova_assets=debug,nova_core=debug,nova_debug=debug"
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
    commands.spawn((status_bar(StatusBarRootConfig::default()),));

    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: Some(game_assets.fps_icon.clone()),
        value_fn: status_fps_value_fn(),
        color_fn: status_fps_color_fn(),
        prefix: "".to_string(),
        suffix: "fps".to_string(),
    }),));
    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: status_version_value_fn(
            std::env::var("APP_VERSION").unwrap_or_else(|_| "unknown".to_string()),
        ),
        color_fn: status_version_color_fn(),
        prefix: "v".to_string(),
        suffix: "".to_string(),
    }),));
}
