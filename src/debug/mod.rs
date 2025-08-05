//! Debugging tools for the game

use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use iyes_perf_ui::prelude::*;

#[derive(Debug, Resource, Default, Clone, Deref, DerefMut)]
struct ShowAxes(pub bool);

/// System set for the debug plugin
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DebugSet;

/// This plugin adds a simple debug system that toggles the Perf UI
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            // we want Bevy to measure these values for us:
            .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
            .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
            .insert_resource(ShowAxes(true))
            .add_plugins(PerfUiPlugin)
            // Bevy egui inspector
            .add_plugins(EguiPlugin::default())
            .add_plugins(WorldInspectorPlugin::new())
            // We need to order our system before PerfUiSet::Setup,
            // so that iyes_perf_ui can process any new Perf UI in the same
            // frame as we spawn the entities. Otherwise, Bevy UI will complain.
            .add_systems(Update, toggle.before(iyes_perf_ui::PerfUiSet::Setup))
            .add_systems(Update, (draw_axes, draw_cursor, add_ui_border))
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    // create a simple Perf UI with default settings
    // and all entries provided by the crate:
    commands.spawn((Name::new("PerfUI"), PerfUiAllEntries::default()));
}

fn toggle(
    mut commands: Commands,
    q_root: Query<Entity, With<PerfUiRoot>>,
    kbd: Res<ButtonInput<KeyCode>>,
    mut show_axes: ResMut<ShowAxes>,
) {
    if kbd.just_pressed(KeyCode::F12) {
        if let Ok(e) = q_root.single() {
            // despawn the existing Perf UI
            commands.entity(e).despawn();
        } else {
            // create a simple Perf UI with default settings
            // and all entries provided by the crate:
            commands.spawn((Name::new("PerfUI"), PerfUiAllEntries::default()));
        }

        show_axes.0 = !show_axes.0;
    }
}

// This system draws the axes based on the cube's transform, with length based on the size of
// the entity's axis-aligned bounding box (AABB).
fn draw_axes(mut gizmos: Gizmos, query: Query<&Transform>, show_axes: Res<ShowAxes>) {
    if !show_axes.0 {
        return;
    }

    for &transform in &query {
        let length = 3.0;
        gizmos.axes(transform, length);
    }
}

fn draw_cursor(
    q_camera: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let Ok((camera, camera_transform)) = q_camera.single() else {
        return;
    };

    let Some(cursor_position) = windows.single().unwrap().cursor_position() else {
        return;
    };

    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else {
        return;
    };
    let point = ray.get_point(distance);

    gizmos.cross(point + Vec3::Y * 0.01, 0.5, Color::WHITE);
}

fn add_ui_border(mut commands: Commands, q_node: Query<Entity, (With<Node>, Without<Outline>)>) {
    for entity in q_node.iter() {
        commands
            .entity(entity)
            .insert(Outline::new(Val::Px(1.0), Val::Px(0.0), Color::WHITE));
    }
}
