use bevy::prelude::*;

pub struct PhysicsDebugPlugin;

impl Plugin for PhysicsDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(avian3d::prelude::PhysicsDebugPlugin::default());
        app.add_systems(Update, update_physics_gizmos);
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
