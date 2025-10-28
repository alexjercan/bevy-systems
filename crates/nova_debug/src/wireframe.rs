use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};

pub struct WireframeDebugPlugin;

impl Plugin for WireframeDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin::default());
        app.insert_resource(WireframeConfig {
            global: true,
            ..default()
        });

        app.add_systems(Update, (toggle_wireframe,));
    }
}

fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    debug: Res<super::DebugEnabled>,
) {
    if debug.is_changed() {
        wireframe_config.global = **debug;
    }
}
