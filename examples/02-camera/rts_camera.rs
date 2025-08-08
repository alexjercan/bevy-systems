//! TODO: Camera example docs

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use systems::camera::rts_camera::prelude::*;

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum CameraMovement {
    #[actionlike(Axis)]
    Zoom,
    #[actionlike(DualAxis)]
    Pan,
    HoldPan,
    HoldOrbit,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RTSCameraPlugin)
        .add_plugins(InputManagerPlugin::<CameraMovement>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, input)
        .configure_sets(Update, RTSCameraSet)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        RTSCamera::default(),
        RTSCameraInput::default(),
        InputMap::default()
            .with_axis(CameraMovement::Zoom, MouseScrollAxis::Y)
            .with_dual_axis(CameraMovement::Pan, MouseMove::default())
            .with(CameraMovement::HoldOrbit, MouseButton::Right)
            .with(CameraMovement::HoldPan, MouseButton::Middle),
        Camera3d::default(),
        Transform::from_xyz(-15.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn input(mut q_camera: Query<(&mut RTSCameraInput, &ActionState<CameraMovement>)>) {
    for (mut input, action) in q_camera.iter_mut() {
        input.pan = Vec2::ZERO;
        input.orbit = Vec2::ZERO;

        if action.pressed(&CameraMovement::HoldOrbit) {
            input.orbit = action.axis_pair(&CameraMovement::Pan);
        } else if action.pressed(&CameraMovement::HoldPan) {
            input.pan = action.axis_pair(&CameraMovement::Pan);
        }

        input.zoom = action.value(&CameraMovement::Zoom);
    }
}
