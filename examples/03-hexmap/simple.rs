//! TODO: Hexmap coordinates docs

#[path = "../helpers/wasd_camera_controller.rs"]
mod wasd_camera_controller;

use bevy::prelude::*;
use hexx::*;

use systems::{debug::DebugPlugin, hexmap::prelude::*};

use wasd_camera_controller::{WASDCameraControllerBundle, WASDCameraControllerPlugin};

const HEX_SIZE: f32 = 1.0;
const CHUNK_RADIUS: u32 = 4;
const DISCOVER_RADIUS: u32 = 3;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HexMapPlugin::new(
            HexLayout::flat().with_hex_size(HEX_SIZE),
            CHUNK_RADIUS,
            DISCOVER_RADIUS,
        ))
        .add_plugins(WASDCameraControllerPlugin)
        .add_plugins(DebugPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, input)
        .configure_sets(Update, HexMapSet)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        WASDCameraControllerBundle::default(),
        Camera3d::default(),
        Transform::from_xyz(60.0, 60.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("RTS Camera"),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(60.0, 60.0, 00.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Directional Light"),
    ));
}

fn input(
    windows: Query<&Window>,
    q_camera: Single<(&Camera, &GlobalTransform)>,
    mut ev_discover: EventWriter<HexDiscoverEvent>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let (camera, transform) = q_camera.into_inner();

    let Some(cursor_position) = windows.single().unwrap().cursor_position() else {
        return;
    };

    let Ok(ray) = camera.viewport_to_world(transform, cursor_position) else {
        return;
    };

    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else {
        return;
    };
    let point = ray.get_point(distance);

    ev_discover.write(HexDiscoverEvent(point.xz()));
}
