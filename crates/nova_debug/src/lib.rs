//! A Bevy plugin that adds various debugging tools.

use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};

pub mod prelude {
    pub use super::gizmos::DebugAxisMarker;
    pub use super::DebugPlugin;
}

/// The keycode to toggle debug mode.
pub const DEBUG_TOGGLE_KEYCODE: KeyCode = KeyCode::F11;

/// Resource with debug toggle state.
#[derive(Resource, Default, Clone, Debug, Deref, DerefMut, PartialEq, Eq, Hash)]
pub struct DebugEnabled(pub bool);

/// A plugin that adds various debugging tools.
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugEnabled(true));

        app.add_plugins(inspector::InpsectorDebugPlugin);
        app.add_plugins(gizmos::DebugGizmosPlugin);
        app.add_plugins(turret::DebugTurretSectionPlugin);
        app.add_plugins(avian3d::prelude::PhysicsDebugPlugin::default());

        app.add_systems(Update, (toggle_debug_mode, update_physics_gizmos));
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

mod inspector {
    use super::*;

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
                    inspector_ui.run_if(resource_equals(DebugEnabled(true))),
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
                    bevy_inspector_egui::bevy_inspector::ui_for_assets::<StandardMaterial>(
                        world, ui,
                    );
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
}

// TODO: Come up with a better name / structure for this plugin
mod gizmos {
    use super::*;

    /// A plugin that draws debug gizmos for entities.
    pub struct DebugGizmosPlugin;

    impl Plugin for DebugGizmosPlugin {
        fn build(&self, app: &mut App) {
            app.add_plugins(WireframePlugin::default());
            app.insert_resource(WireframeConfig {
                global: true,
                ..default()
            });

            app.add_systems(
                Update,
                (
                    draw_debug_gizmos_axis.run_if(resource_equals(DebugEnabled(true))),
                    toggle_wireframe,
                ),
            );
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
}

mod turret {
    use super::*;
    use nova_gameplay::prelude::*;

    pub struct DebugTurretSectionPlugin;

    impl Plugin for DebugTurretSectionPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(
                Update,
                (debug_draw_barrel_direction, debug_gizmos_turret_forward)
                    .run_if(resource_equals(DebugEnabled(true))),
            );
        }
    }

    const DEBUG_LINE_LENGTH: f32 = 100.0;

    fn debug_draw_barrel_direction(
        q_muzzle: Query<&TransformChainWorld, With<TurretSectionBarrelMuzzleMarker>>,
        mut gizmos: Gizmos,
    ) {
        for barrel_transform in &q_muzzle {
            let barrel_pos = barrel_transform.translation;
            let barrel_dir = barrel_transform.rotation * Vec3::NEG_Z;

            let line_length = DEBUG_LINE_LENGTH;
            let line_end = barrel_pos + barrel_dir * line_length;

            gizmos.line(barrel_pos, line_end, Color::srgb(1.0, 0.0, 0.0));
        }
    }

    fn debug_gizmos_turret_forward(
        mut gizmos: Gizmos,
        q_turret: Query<(&GlobalTransform, &TurretSectionTargetInput), With<TurretSectionMarker>>,
    ) {
        for (transform, target) in &q_turret {
            if let Some(target) = **target {
                let origin = transform.translation();
                let dir = (target - origin).normalize() * DEBUG_LINE_LENGTH;
                gizmos.line(origin, origin + dir, Color::srgb(0.9, 0.9, 0.1));
            }
        }
    }
}
