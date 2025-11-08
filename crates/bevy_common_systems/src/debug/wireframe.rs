use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};

/// The keycode to toggle debug mode.
pub const DEBUG_TOGGLE_KEYCODE: KeyCode = KeyCode::F11;

/// Resource with debug toggle state.
#[derive(Resource, Default, Clone, Debug, Deref, DerefMut, PartialEq, Eq, Hash)]
pub struct DebugEnabled(pub bool);

pub struct WireframeDebugPlugin;

impl Plugin for WireframeDebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugEnabled(true));

        app.add_plugins(WireframePlugin::default());
        app.insert_resource(WireframeConfig {
            global: true,
            ..default()
        });

        app.add_systems(Update, (enable_wireframe, toggle_debug_mode));
    }
}

fn enable_wireframe(mut wireframe_config: ResMut<WireframeConfig>, debug: Res<DebugEnabled>) {
    if debug.is_changed() {
        wireframe_config.global = **debug;
    }
}

fn toggle_debug_mode(mut debug: ResMut<DebugEnabled>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(DEBUG_TOGGLE_KEYCODE) {
        **debug = !**debug;
    }
}
