//! TODO: Hexmap coordinates docs

use std::time::SystemTime;

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use systems::{
    camera::wasd_camera::{WASDCamera, WASDCameraInput, WASDCameraPlugin, WASDCameraSet},
    debug::DebugPlugin,
    hexmap::map::{GeneratorKind, HexDiscoverEvent, HexMapPlugin, HexMapSet},
};

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum CameraMovement {
    #[actionlike(DualAxis)]
    Pan,
    #[actionlike(DualAxis)]
    Wasd,
    #[actionlike(Axis)]
    Vertical,
    HoldPan,
    LeftClick,
}

const CHUNK_RADIUS: u32 = 4;
const HEX_SIZE: f32 = 1.0;
const DISCOVER_RADIUS: u32 = 3;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HexMapPlugin::new(
            GeneratorKind::Perlin(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u32,
            ),
            HEX_SIZE,
            CHUNK_RADIUS,
            DISCOVER_RADIUS,
        ))
        .add_plugins(WASDCameraPlugin)
        .add_plugins(InputManagerPlugin::<CameraMovement>::default())
        .add_plugins(DebugPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, input)
        .configure_sets(Update, WASDCameraSet)
        .configure_sets(Update, HexMapSet)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        WASDCamera::default(),
        WASDCameraInput::default(),
        InputMap::default()
            .with_dual_axis(CameraMovement::Pan, MouseMove::default())
            .with_dual_axis(CameraMovement::Wasd, VirtualDPad::wasd())
            .with_axis(
                CameraMovement::Vertical,
                VirtualAxis::new(KeyCode::ShiftLeft, KeyCode::Space),
            )
            .with(CameraMovement::HoldPan, MouseButton::Right)
            .with(CameraMovement::LeftClick, MouseButton::Left),
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
    mut q_camera: Query<(
        &Camera,
        &GlobalTransform,
        &mut WASDCameraInput,
        &ActionState<CameraMovement>,
    )>,
    mut ev_discover: EventWriter<HexDiscoverEvent>,
) {
    for (camera, transform, mut input, action) in q_camera.iter_mut() {
        input.pan = Vec2::ZERO;

        if action.pressed(&CameraMovement::HoldPan) {
            input.pan = action.axis_pair(&CameraMovement::Pan);
        }

        input.wasd = action.axis_pair(&CameraMovement::Wasd);
        input.vertical = action.value(&CameraMovement::Vertical);

        if action.just_pressed(&CameraMovement::LeftClick) {
            let Some(cursor_position) = windows.single().unwrap().cursor_position() else {
                return;
            };

            let Ok(ray) = camera.viewport_to_world(transform, cursor_position) else {
                return;
            };

            let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
            else {
                return;
            };
            let point = ray.get_point(distance);

            ev_discover.write(HexDiscoverEvent(point.xz()));
        }
    }
}
