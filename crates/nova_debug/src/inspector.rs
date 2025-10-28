use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui,
    bevy_egui::{EguiContext, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext},
    egui, DefaultInspectorConfigPlugin,
};

/// A plugin that adds an inspector UI for debugging.
pub struct InpsectorDebugPlugin;

impl Plugin for InpsectorDebugPlugin {
    fn build(&self, app: &mut App) {
        app
            // Bevy egui inspector
            .add_plugins(EguiPlugin::default())
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_systems(
                EguiPrimaryContextPass,
                inspector_ui.run_if(resource_equals(super::DebugEnabled(true))),
            );

        app.insert_resource(bevy_egui::EguiGlobalSettings {
            auto_create_primary_context: false,
            ..Default::default()
        });

        // TODO: Ideally we would have an extra camera for the inspector only, but for now we
        // will just use the primary camera.
        app.add_observer(on_add_camera);
    }
}

fn inspector_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world)
    else {
        warn!("inspector_ui: No EguiContext found");
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new("UI").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // equivalent to `WorldInspectorPlugin`
            bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);

            egui::CollapsingHeader::new("Materials").show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::ui_for_assets::<StandardMaterial>(world, ui);
            });

            ui.heading("Entities");
            bevy_inspector_egui::bevy_inspector::ui_for_entities(world, ui);
        });
    });
}

fn on_add_camera(add: On<Add, Camera>, mut commands: Commands) {
    let entity = add.entity;
    debug!(
        "Camera added to entity {:?}, inserting PrimaryEguiContext to it.",
        entity
    );

    commands.entity(entity).insert(PrimaryEguiContext);
}
