use bevy::prelude::*;
use avian3d::prelude::*;

pub struct PhysicsDebugPlugin;

impl Plugin for PhysicsDebugPlugin {
    fn build(&self, app: &mut App) {
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

        app.add_systems(Update, (update_physics_gizmos, update_physics_ui));
    }
}

fn update_physics_gizmos(debug: Res<super::DebugEnabled>, mut store: ResMut<GizmoConfigStore>) {
    if debug.is_changed() {
        store
            .config_mut::<avian3d::prelude::PhysicsGizmos>()
            .0
            .enabled = **debug;
    }
}

fn update_physics_ui(
    debug: Res<super::DebugEnabled>,
    mut settings: ResMut<PhysicsDiagnosticsUiSettings>,
) {
    if debug.is_changed() {
        settings.enabled = **debug;
    }
}
