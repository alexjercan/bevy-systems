//! WASD camera controller using bevy_enhanced_input for input handling.
//!
//! This should be used together with the [`WASDCamera`] component from the `bevy_common_systems`
//! crate. It sets up input bindings for WASD movement, mouse look, and vertical movement
//! (space and shift keys).

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::prelude::{WASDCamera, WASDCameraInput};

pub mod prelude {
    pub use super::{
        WASDCameraController, WASDCameraControllerPlugin, WASDCameraControllerSystems,
    };
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum WASDCameraControllerSystems {
    Sync,
}

/// A plugin that sets up WASD and mouse controls for a camera.
pub struct WASDCameraControllerPlugin;

impl Plugin for WASDCameraControllerPlugin {
    fn build(&self, app: &mut App) {
        debug!("WASDCameraControllerPlugin: build");

        app.add_input_context::<WASDCameraInputMarker>();

        app.add_observer(setup_wasd_camera);
        app.add_observer(destroy_wasd_camera);

        app.add_observer(on_wasd_input);
        app.add_observer(on_wasd_input_completed);
        app.add_observer(on_mouse_input);
        app.add_observer(on_mouse_input_completed);
        app.add_observer(on_enable_look_input);
        app.add_observer(on_enable_look_input_completed);
        app.add_observer(on_vertical_input);
        app.add_observer(on_vertical_input_completed);
    }
}

#[derive(Component, Debug, Clone)]
struct WASDCameraInputMarker;

#[derive(InputAction)]
#[action_output(Vec2)]
struct WASDCameraInputMove;

#[derive(InputAction)]
#[action_output(Vec2)]
struct WASDCameraInputLook;

#[derive(InputAction)]
#[action_output(bool)]
struct WASDCameraInputEnableLook;

#[derive(InputAction)]
#[action_output(f32)]
struct WASDCameraInputVertical;

#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut, Reflect)]
struct WASDCameraLookEnabled(bool);

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
pub struct WASDCameraController;

fn setup_wasd_camera(insert: On<Insert, WASDCameraController>, mut commands: Commands) {
    let entity = insert.entity;
    trace!("setup_wasd_camera: entity {:?}", entity);

    commands.entity(entity).insert((
        Camera3d::default(),
        WASDCamera {
            wasd_sensitivity: 0.1,
            ..default()
        },
        WASDCameraLookEnabled(false),
        WASDCameraInputMarker,
        actions!(
            WASDCameraInputMarker[
                (
                    Name::new("Input: WASD Camera Move"),
                    Action::<WASDCameraInputMove>::new(),
                    Bindings::spawn((
                        Cardinal::wasd_keys().with(Scale::splat(1.0)),
                        Axial::left_stick().with(Scale::splat(1.0)),
                    )),
                ),
                (
                    Name::new("Input: WASD Camera Look"),
                    Action::<WASDCameraInputLook>::new(),
                    Bindings::spawn((
                        // Bevy requires single entities to be wrapped in `Spawn`.
                        // You can attach modifiers to individual bindings as well.
                        Spawn((Binding::mouse_motion(), Scale::splat(0.01), Negate::none())),
                        Axial::right_stick().with((Scale::splat(1.0), Negate::none())),
                    )),
                ),
                (
                    Name::new("Input: WASD Camera Enable Look"),
                    Action::<WASDCameraInputEnableLook>::new(),
                    bindings![MouseButton::Right],
                ),
                (
                    Name::new("Input: WASD Camera Vertical"),
                    Action::<WASDCameraInputVertical>::new(),
                    Bindings::spawn((
                        Bidirectional::<Binding, Binding> {
                            positive: KeyCode::Space.into(),
                            negative: KeyCode::ShiftLeft.into(),
                        },
                    )),
                ),
            ]
        ),
    ));
}

fn destroy_wasd_camera(remove: On<Remove, WASDCameraController>, mut commands: Commands) {
    let entity = remove.entity;
    trace!("destroy_wasd_camera: entity {:?}", entity);

    // use try_remove in case this get's despawned and remove is called after
    commands.entity(entity).try_remove::<(
        Actions<WASDCameraInputMarker>,
        WASDCamera,
        WASDCameraLookEnabled,
        WASDCameraInputMarker,
    )>();
}

fn on_wasd_input(fire: On<Fire<WASDCameraInputMove>>, mut q_input: Query<&mut WASDCameraInput>) {
    for mut input in &mut q_input {
        input.wasd = fire.value;
    }
}

fn on_wasd_input_completed(
    _: On<Complete<WASDCameraInputMove>>,
    mut q_input: Query<&mut WASDCameraInput>,
) {
    for mut input in &mut q_input {
        input.wasd = Vec2::ZERO;
    }
}

fn on_mouse_input(
    fire: On<Fire<WASDCameraInputLook>>,
    mut q_input: Query<(&mut WASDCameraInput, &WASDCameraLookEnabled)>,
) {
    for (mut input, enabled) in &mut q_input {
        if !**enabled {
            continue;
        }

        input.pan = fire.value;
    }
}

fn on_mouse_input_completed(
    _: On<Complete<WASDCameraInputLook>>,
    mut q_input: Query<&mut WASDCameraInput>,
) {
    for mut input in &mut q_input {
        input.pan = Vec2::ZERO;
    }
}

fn on_enable_look_input(
    _: On<Fire<WASDCameraInputEnableLook>>,
    mut q_look_enabled: Query<&mut WASDCameraLookEnabled>,
) {
    for mut look_enabled in &mut q_look_enabled {
        **look_enabled = true;
    }
}

fn on_enable_look_input_completed(
    _: On<Complete<WASDCameraInputEnableLook>>,
    mut q_look_enabled: Query<(&mut WASDCameraInput, &mut WASDCameraLookEnabled)>,
) {
    for (mut input, mut look_enabled) in &mut q_look_enabled {
        input.pan = Vec2::ZERO;
        **look_enabled = false;
    }
}

fn on_vertical_input(
    fire: On<Fire<WASDCameraInputVertical>>,
    mut q_input: Query<&mut WASDCameraInput>,
) {
    for mut input in &mut q_input {
        input.vertical = fire.value;
    }
}

fn on_vertical_input_completed(
    _: On<Complete<WASDCameraInputVertical>>,
    mut q_input: Query<&mut WASDCameraInput>,
) {
    for mut input in &mut q_input {
        input.vertical = 0.0;
    }
}
