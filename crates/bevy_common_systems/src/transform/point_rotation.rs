use bevy::prelude::*;

pub mod prelude {
    pub use super::PointRotation;
    pub use super::PointRotationInput;
    pub use super::PointRotationOutput;
    pub use super::PointRotationPlugin;
    pub use super::PointRotationSystems;
}

/// Component that marks an entity as a point that can be rotated.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct PointRotation {
    /// Initial rotation of the point - used to compute the initial facing direction.
    pub initial_rotation: Quat,
}

impl Default for PointRotation {
    fn default() -> Self {
        Self {
            // We assume the initial rotation is identity, meaning the point is facing down the -Z
            // axis, up is +Y, and right is +X.
            initial_rotation: Quat::IDENTITY,
        }
    }
}

/// The delta by how much to rotate the point
#[derive(Component, Default, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct PointRotationInput(pub Vec2);

/// The target rotation state of the point, which is computed from the State and the Input.
#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut, Reflect)]
pub struct PointRotationOutput(pub Quat);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PointRotationSystems {
    Sync,
}

/// A plugin that will enable the PointRotation system.
///
/// PointRotation allows an entity to be rotated based on input deltas, typically from mouse
/// movement. The rotation is applied around the entity's local axes, allowing for intuitive
/// control of the entity's orientation.
pub struct PointRotationPlugin;

impl Plugin for PointRotationPlugin {
    fn build(&self, app: &mut App) {
        debug!("PointRotationPlugin: build");

        app.add_observer(initialize_point_rotation_system);

        app.add_systems(
            PostUpdate,
            point_rotation_update_system.in_set(PointRotationSystems::Sync),
        );
    }
}

fn initialize_point_rotation_system(
    insert: On<Insert, PointRotation>,
    mut commands: Commands,
    q_point: Query<&PointRotation, With<PointRotation>>,
) {
    let entity = insert.entity;
    trace!("initialize_point_rotation_system: entity {:?}", entity);

    let Ok(point) = q_point.get(entity) else {
        warn!(
            "initialize_point_rotation_system: entity {:?} not found in q_point",
            entity
        );
        return;
    };

    commands
        .entity(entity)
        .insert(PointRotationInput(Vec2::ZERO))
        .insert(PointRotationOutput(point.initial_rotation));
}

fn point_rotation_update_system(
    mut q_point: Query<(&PointRotationInput, &mut PointRotationOutput), With<PointRotation>>,
) {
    for (input, mut out) in &mut q_point {
        let mut state = PointRotationState::from(**out);
        state = compute_point_rotation(&state, &input);
        **out = point_rotation_quat(state);
    }
}

#[derive(Clone, Copy, Debug)]
struct PointRotationState {
    forward: Vec3,
    right: Vec3,
}

impl From<Quat> for PointRotationState {
    fn from(rotation: Quat) -> Self {
        let forward = rotation * Vec3::NEG_Z;
        let right = rotation * Vec3::X;
        Self { forward, right }
    }
}

fn compute_point_rotation(
    state: &PointRotationState,
    input: &PointRotationInput,
) -> PointRotationState {
    let mut new_state = *state;

    let delta_x = input.x;
    let delta_y = input.y;

    if delta_x != 0.0 {
        let up = new_state.right.cross(new_state.forward).normalize();
        let yaw_quat = Quat::from_axis_angle(up, delta_x);
        new_state.forward = (yaw_quat * new_state.forward).normalize();
        new_state.right = (yaw_quat * new_state.right).normalize();
    }

    if delta_y != 0.0 {
        let right = new_state.right.normalize();
        let pitch_quat = Quat::from_axis_angle(right, delta_y);
        new_state.forward = (pitch_quat * new_state.forward).normalize();
        new_state.right = (pitch_quat * new_state.right).normalize();
    }

    new_state
}

fn point_rotation_quat(state: PointRotationState) -> Quat {
    let forward = state.forward.normalize();
    let right = state.right.normalize();
    let up = state.right.cross(state.forward).normalize();

    // Local basis: right, up, forward
    let mat3 = Mat3::from_cols(right, up, -forward);
    Quat::from_mat3(&mat3)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compute_point_rotation_1() {
        let initial_state = PointRotationState {
            forward: Vec3::NEG_Z,
            right: Vec3::X,
        };

        // Rotate 90 degrees to the right (yaw)
        let input = PointRotationInput(Vec2::new(-std::f32::consts::FRAC_PI_2, 0.0));
        let new_state = compute_point_rotation(&initial_state, &input);
        assert!(new_state.forward.abs_diff_eq(Vec3::X, 1e-6));
        assert!(new_state.right.abs_diff_eq(Vec3::Z, 1e-6));
    }

    #[test]
    fn test_compute_point_rotation_2() {
        let initial_state = PointRotationState {
            forward: Vec3::NEG_Z,
            right: Vec3::X,
        };

        // Rotate 90 degrees up (pitch)
        let input = PointRotationInput(Vec2::new(0.0, std::f32::consts::FRAC_PI_2));
        let new_state = compute_point_rotation(&initial_state, &input);
        assert!(new_state.forward.abs_diff_eq(Vec3::Y, 1e-6));
        assert!(new_state.right.abs_diff_eq(Vec3::X, 1e-6));
    }

    #[test]
    fn test_compute_point_rotation_3() {
        let initial_state = PointRotationState {
            forward: Vec3::NEG_Z,
            right: Vec3::X,
        };

        // Rotate 45 degrees right and 45 degrees up
        let input = PointRotationInput(Vec2::new(
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_4,
        ));
        let new_state = compute_point_rotation(&initial_state, &input);
        let expected_forward = Vec3::new(0.5, 0.70710677, -0.5).normalize();
        let expected_right = Vec3::new(0.70710677, 0.0, 0.70710677).normalize();
        assert!(new_state.forward.abs_diff_eq(expected_forward, 1e-6));
        assert!(new_state.right.abs_diff_eq(expected_right, 1e-6));
    }

    #[test]
    fn test_compute_point_rotation_4() {
        let initial_state = PointRotationState {
            forward: Vec3::NEG_Z,
            right: Vec3::X,
        };

        // Rotate 90 degrees to the left (yaw)
        let input = PointRotationInput(Vec2::new(std::f32::consts::FRAC_PI_2, 0.0));
        let new_state = compute_point_rotation(&initial_state, &input);
        assert!(new_state.forward.abs_diff_eq(Vec3::NEG_X, 1e-6));
        assert!(new_state.right.abs_diff_eq(Vec3::NEG_Z, 1e-6));
    }

    #[test]
    fn test_compute_point_rotation_5() {
        let initial_state = PointRotationState {
            forward: Vec3::NEG_Z,
            right: Vec3::X,
        };

        // Rotate -90 degrees down (pitch)
        let input = PointRotationInput(Vec2::new(0.0, -std::f32::consts::FRAC_PI_2));
        let new_state = compute_point_rotation(&initial_state, &input);
        assert!(new_state.forward.abs_diff_eq(Vec3::NEG_Y, 1e-6));
        assert!(new_state.right.abs_diff_eq(Vec3::X, 1e-6));
    }
}
