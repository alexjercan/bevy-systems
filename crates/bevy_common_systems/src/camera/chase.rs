//! A Bevy plugin to implement a chase camera that follows a target entity with smoothing and
//! offset.

use bevy::prelude::*;

use crate::prelude::LerpSnap;

pub mod prelude {
    pub use super::{ChaseCamera, ChaseCameraInput, ChaseCameraPlugin, ChaseCameraSystems};
}

/// The Case Camera Component is used to add some attributes to the camera
///
/// The Case Camera will follow a target and lag behind it a bit to give a smooth transition and
/// give a perspective of movement. We also want the camera to look in front of the target, so we
/// add a focus offset point. The Chase Camera will also have a rotation orbit style to look around
/// the focus point at the target.
#[derive(Component, Debug, Reflect)]
#[require(Transform)]
pub struct ChaseCamera {
    /// Offset distance behind the target
    pub offset: Vec3,
    /// How far ahead of the target to look
    pub focus_offset: Vec3,
    /// Smoothing factor for camera movement (0.0 = no smoothing, 1.0 = very smooth)
    pub smoothing: f32,
}

impl Default for ChaseCamera {
    fn default() -> Self {
        Self {
            offset: Vec3::new(0.0, 5.0, -20.0),
            focus_offset: Vec3::new(0.0, 0.0, 20.0),
            smoothing: 0.0,
        }
    }
}

/// The anchor point for the chase camera to use as a frame of reference
#[derive(Component, Default, Debug, Reflect)]
pub struct ChaseCameraInput {
    /// The rotation that we want to match
    pub achor_rot: Quat,
    /// The position that we want to match (and we will add the offset to this)
    pub anchor_pos: Vec3,
}

#[derive(Component, Default, Debug, Reflect)]
struct ChaseCameraState {
    anchor_pos: Vec3,
}

/// The SystemSet for the ChaseCameraPlugin
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChaseCameraSystems {
    Sync,
}

/// Plugin to manage entities with `ChaseCamera` component.
pub struct ChaseCameraPlugin;

impl Plugin for ChaseCameraPlugin {
    fn build(&self, app: &mut App) {
        debug!("ChaseCameraPlugin: build");

        app.add_observer(initialize_chase_camera);
        app.add_observer(destroy_chase_camera);

        // NOTE: I am using PostUpdate here to ensure that the camera updates after the input was
        // set by the user or other systems in the Update stage. Then the new transform will be
        // available in the next frame's Update stage.
        app.add_systems(
            PostUpdate,
            (
                chase_camera_update_state_system,
                chase_camera_sync_transform_system,
            )
                .chain()
                .in_set(ChaseCameraSystems::Sync),
        );
    }
}

/// Initialize camera state and target from config
fn initialize_chase_camera(
    insert: On<Insert, ChaseCamera>,
    mut commands: Commands,
    q_state: Query<Has<ChaseCameraState>, With<ChaseCamera>>,
) {
    let entity = insert.entity;
    trace!("initialize_chase_camera: entity {:?}", entity);

    let Ok(has_state) = q_state.get(entity) else {
        warn!(
            "initialize_chase_camera: entity {:?} not found in q_state",
            entity
        );
        return;
    };

    commands
        .entity(entity)
        .insert((ChaseCameraInput::default(),));

    if !has_state {
        commands.entity(entity).insert(ChaseCameraState::default());
    }
}

fn destroy_chase_camera(remove: On<Remove, ChaseCamera>, mut commands: Commands) {
    let entity = remove.entity;
    trace!("destroy_chase_camera: entity {:?}", entity);

    // NOTE: use try_remove in case this get's despawned and remove is called after
    commands
        .entity(entity)
        .try_remove::<(ChaseCameraInput, ChaseCameraState)>();
}

fn chase_camera_update_state_system(
    time: Res<Time>,
    mut q_camera: Query<
        (&ChaseCamera, &ChaseCameraInput, &mut ChaseCameraState),
        With<ChaseCamera>,
    >,
) {
    let dt = time.delta_secs();

    for (chase, input, mut state) in q_camera.iter_mut() {
        let target_pos = input.anchor_pos;
        let desired_pos = target_pos
            + input.achor_rot * Vec3::NEG_Z * chase.offset.z
            + input.achor_rot * Vec3::Y * chase.offset.y
            + input.achor_rot * Vec3::X * chase.offset.x;

        state.anchor_pos = state
            .anchor_pos
            .lerp_and_snap(desired_pos, chase.smoothing, dt);
    }
}

fn chase_camera_sync_transform_system(
    mut q_camera: Query<
        (
            &ChaseCamera,
            &ChaseCameraInput,
            &ChaseCameraState,
            &mut Transform,
        ),
        With<ChaseCamera>,
    >,
) {
    for (chase, input, state, mut transform) in q_camera.iter_mut() {
        transform.translation = state.anchor_pos;

        let focus = input.anchor_pos
            + input.achor_rot * Vec3::NEG_Z * chase.focus_offset.z
            + input.achor_rot * Vec3::Y * chase.focus_offset.y
            + input.achor_rot * Vec3::X * chase.focus_offset.x;

        transform.look_at(focus, input.achor_rot * Vec3::Y);
    }
}
