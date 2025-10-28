//! A Bevy plugin that adds various debugging tools.

use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};

pub mod spawner;
pub mod inspector;
pub mod wireframe;
pub mod turret;

pub mod prelude {
    pub use super::DebugPlugin;
}

/// The keycode to toggle debug mode.
pub const DEBUG_TOGGLE_KEYCODE: KeyCode = KeyCode::F11;

/// Resource with debug toggle state.
#[derive(Resource, Default, Clone, Debug, Deref, DerefMut, PartialEq, Eq, Hash)]
pub struct DebugEnabled(pub bool);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DebugSystems;

/// A plugin that adds various debugging tools.
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugEnabled(true));

        app.add_plugins(inspector::InpsectorDebugPlugin);
        app.add_plugins(spawner::SpawnerPlugin);
        app.add_plugins(wireframe::WireframeDebugPlugin);
        app.add_plugins(turret::DebugTurretSectionPlugin);

        app.add_plugins(avian3d::prelude::PhysicsDebugPlugin::default());

        app.edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        });

        app.add_systems(Update, (toggle_debug_mode, update_physics_gizmos));

        app.configure_sets(
            Update,
            DebugSystems.run_if(resource_equals(DebugEnabled(true))),
        );
    }
}

fn toggle_debug_mode(mut debug: ResMut<DebugEnabled>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(DEBUG_TOGGLE_KEYCODE) {
        **debug = !**debug;
    }
}

fn update_physics_gizmos(debug: Res<DebugEnabled>, mut store: ResMut<GizmoConfigStore>) {
    if debug.is_changed() {
        store
            .config_mut::<avian3d::prelude::PhysicsGizmos>()
            .0
            .enabled = **debug;
    }
}
