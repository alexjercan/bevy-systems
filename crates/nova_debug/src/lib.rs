use bevy::{pbr::wireframe::{WireframeConfig, WireframePlugin}, prelude::*};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext},
    bevy_inspector, egui, DefaultInspectorConfigPlugin,
};

pub mod prelude {
    pub use super::DebugPlugin;
    pub use super::DebugAxisMarker;
}

/// The keycode to toggle debug mode.
pub const DEBUG_TOGGLE_KEYCODE: KeyCode = KeyCode::F11;

/// A plugin that adds various debugging tools.
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InpsectorDebugPlugin, DebugGizmosPlugin));
        app.add_plugins(avian3d::prelude::PhysicsDebugPlugin::default());
    }
}

/// A plugin that adds an inspector UI for debugging.
pub struct InpsectorDebugPlugin;

impl Plugin for InpsectorDebugPlugin {
    fn build(&self, app: &mut App) {
        app
            // Bevy egui inspector
            .add_plugins(EguiPlugin::default())
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_systems(EguiPrimaryContextPass, inspector_ui);
    }
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

/// A plugin that draws debug gizmos for entities.
pub struct DebugGizmosPlugin;

impl Plugin for DebugGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin::default());
        app.insert_resource(WireframeConfig {
            global: true,
            ..default()
        });

        app.add_systems(Update, (draw_debug_gizmos_axis, toggle_wireframe));
    }
}

/// Entities with this component will have their local axes drawn in the world.
#[derive(Component)]
pub struct DebugAxisMarker;

fn draw_debug_gizmos_axis(
    mut gizmos: Gizmos,
    q_transform: Query<&GlobalTransform, With<DebugAxisMarker>>,
) {
    // Draw the xyz axis of all entities with a GlobalTransform
    for transform in &q_transform {
        let origin = transform.translation();
        let x_axis = transform.rotation() * Vec3::X * 2.0;
        let y_axis = transform.rotation() * Vec3::Y * 2.0;
        let z_axis = transform.rotation() * Vec3::NEG_Z * 2.0;

        gizmos.line(origin, origin + x_axis, Color::srgb(0.9, 0.1, 0.1));
        gizmos.line(origin, origin + y_axis, Color::srgb(0.1, 0.9, 0.1));
        gizmos.line(origin, origin + z_axis, Color::srgb(0.1, 0.1, 0.9));
    }
}

fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(DEBUG_TOGGLE_KEYCODE) {
        wireframe_config.global = !wireframe_config.global;
    }
}
