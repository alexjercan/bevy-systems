use crate::camera::wasd_camera::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub mod prelude {
    pub use super::{
        WASDCameraControllerBundle, WASDCameraControllerPlugin, WASDCameraControllerPluginSet,
    };
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum CameraMovement {
    #[actionlike(DualAxis)]
    Pan,
    #[actionlike(DualAxis)]
    Wasd,
    #[actionlike(Axis)]
    Vertical,
    HoldPan,
}

#[derive(Bundle, Clone, Debug)]
pub struct WASDCameraControllerBundle {
    camera: WASDCamera,
    input: WASDCameraInput,
    map: InputMap<CameraMovement>,
}

impl Default for WASDCameraControllerBundle {
    fn default() -> Self {
        Self {
            camera: WASDCamera {
                wasd_sensitivity: 2.0,
                ..default()
            },
            input: WASDCameraInput::default(),
            map: InputMap::default()
                .with_dual_axis(CameraMovement::Pan, MouseMove::default())
                .with_dual_axis(CameraMovement::Wasd, VirtualDPad::wasd())
                .with_axis(
                    CameraMovement::Vertical,
                    VirtualAxis::new(KeyCode::ShiftLeft, KeyCode::Space),
                )
                .with(CameraMovement::HoldPan, MouseButton::Right),
        }
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WASDCameraControllerPluginSet;

pub struct WASDCameraControllerPlugin;

impl Plugin for WASDCameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WASDCameraPlugin);
        app.add_plugins(InputManagerPlugin::<CameraMovement>::default());
        app.configure_sets(Update, WASDCameraSet);

        app.add_systems(Update, input.in_set(WASDCameraControllerPluginSet));
    }
}

fn input(mut q_camera: Query<(&mut WASDCameraInput, &ActionState<CameraMovement>)>) {
    for (mut input, action) in q_camera.iter_mut() {
        input.pan = Vec2::ZERO;

        if action.pressed(&CameraMovement::HoldPan) {
            input.pan = action.axis_pair(&CameraMovement::Pan);
        }

        input.wasd = action.axis_pair(&CameraMovement::Wasd);
        input.vertical = action.value(&CameraMovement::Vertical);
    }
}
