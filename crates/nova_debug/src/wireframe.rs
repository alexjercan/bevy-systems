use bevy::{pbr::wireframe::{WireframeConfig, WireframePlugin}, prelude::*};

pub struct WireframeDebugPlugin;

impl Plugin for WireframeDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin::default());
        app.insert_resource(WireframeConfig {
            global: true,
            ..default()
        });

        app.add_systems(
            Update,
            (
                toggle_wireframe,
            ),
        );
    }
}

fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // TODO: I would rather not have to hardcode key press here...
    if keyboard.just_pressed(super::DEBUG_TOGGLE_KEYCODE) {
        wireframe_config.global = !wireframe_config.global;
    }
}

