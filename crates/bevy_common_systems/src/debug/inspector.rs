use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui,
    bevy_egui::{EguiContext, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext},
    egui, DefaultInspectorConfigPlugin,
};

/// The keycode to toggle debug mode.
pub const DEBUG_TOGGLE_KEYCODE: KeyCode = KeyCode::F11;

/// Resource with debug toggle state.
#[derive(Resource, Default, Clone, Debug, Deref, DerefMut, PartialEq, Eq, Hash)]
pub struct DebugEnabled(pub bool);

/// A plugin that adds an inspector UI for debugging.
pub struct InpsectorDebugPlugin;

impl Plugin for InpsectorDebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugEnabled(true));

        app
            // Bevy egui inspector
            .add_plugins(EguiPlugin::default())
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_systems(
                EguiPrimaryContextPass,
                inspector_ui.run_if(resource_equals(DebugEnabled(true))),
            );

        app.insert_resource(bevy_egui::EguiGlobalSettings {
            auto_create_primary_context: false,
            ..Default::default()
        });

        app.add_observer(on_add_camera);

        app.add_plugins((
            avian3d::prelude::PhysicsDebugPlugin::default(),
            // Add the `PhysicsDiagnosticsPlugin` to write physics diagnostics
            // to the `DiagnosticsStore` resource in `bevy_diagnostic`.
            // Requires the `bevy_diagnostic` feature.
            PhysicsDiagnosticsPlugin,
            // Add the `PhysicsDiagnosticsUiPlugin` to display physics diagnostics
            // in a debug UI. Requires the `diagnostic_ui` feature.
            PhysicsDiagnosticsUiPlugin,
        ));

        app.add_systems(
            Update,
            (enable_physics_gizmos, enable_physics_ui, toggle_debug_mode),
        );
    }
}

fn inspector_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world)
    else {
        error!("inspector_ui: No EguiContext found");
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
    debug!("on_add_camera: entity {:?}", entity);

    commands.entity(entity).insert(PrimaryEguiContext);
}

fn enable_physics_gizmos(mut store: ResMut<GizmoConfigStore>, debug: Res<DebugEnabled>) {
    if debug.is_changed() {
        store
            .config_mut::<avian3d::prelude::PhysicsGizmos>()
            .0
            .enabled = **debug;
    }
}

fn enable_physics_ui(mut settings: ResMut<PhysicsDiagnosticsUiSettings>, debug: Res<DebugEnabled>) {
    if debug.is_changed() {
        settings.enabled = **debug;
    }
}

fn toggle_debug_mode(mut debug: ResMut<DebugEnabled>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(DEBUG_TOGGLE_KEYCODE) {
        **debug = !**debug;
    }
}
