use avian3d::{math::*, prelude::*};
use bevy::prelude::*;

pub mod prelude {
    pub use super::StableTorquePdController;
    pub use super::StableTorquePdControllerTarget;
    pub use super::StableTorquePdControllerPlugin;
}

#[derive(Component, Debug, Clone, Reflect)]
#[require(RigidBody, Rotation, StableTorquePdControllerTarget)]
/// A Proportional-Derivative controller that applies torque to reach a target angle in a stable
/// manner.
///
/// When we add the `StableTorquePdController` component to an entity, we also need to add the
/// `StableTorquePdControllerTarget` component to specify the desired target rotation. We can then
/// use the target component to update the target rotation as needed. The entity's rotation will be
/// modified by the controller to reach the target rotation.
pub struct StableTorquePdController {
    /// Frequency in Hz.
    pub frequency: f32,
    /// Damping ratio.
    pub damping_ratio: f32,
    /// Maximum torque that can be applied.
    pub max_torque: f32,
}

#[derive(Component, Debug, Clone, Default, Deref, DerefMut, Reflect)]
/// The target rotation for the StableTorquePdController.
///
/// This represents the desired orientation that the controller will attempt to achieve.
/// We can use the Rotation component to represent the current state of the entity.
pub struct StableTorquePdControllerTarget(pub Quaternion);

/// A plugin that will enable the StableTorquePdController.
pub struct StableTorquePdControllerPlugin;

impl Plugin for StableTorquePdControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StableTorquePdController>()
            .register_type::<StableTorquePdControllerTarget>();

        app.add_systems(Update, stable_torque_pd_controller_system);
    }
}

fn stable_torque_pd_controller_system(
    mut q_controller: Query<(
        &mut ExternalTorque,
        &AngularVelocity,
        &ComputedAngularInertia,
        &Rotation,
        &StableTorquePdController,
        &StableTorquePdControllerTarget,
    )>,
) {
    for (mut torque, angular_velocity, angular_inertia, rotation, controller, target) in
        &mut q_controller
    {
        let (principal, local_frame) = angular_inertia.principal_angular_inertia_with_local_frame();

        **torque = compute_pd_torque(
            controller.frequency,
            controller.damping_ratio,
            controller.max_torque,
            **rotation,
            **target,
            **angular_velocity,
            principal,
            local_frame,
        );
    }
}

fn compute_pd_torque(
    frequency: f32,
    damping_ratio: f32,
    max_torque: f32,
    from_rotation: Quaternion,
    to_rotation: Quaternion,
    angular_velocity: Vec3,
    inertia_principal: Vec3,
    inertia_local_frame: Quaternion,
) -> Vector3 {
    // PD gains
    let kp = (6.0 * frequency).powi(2) * 0.25;
    let kd = 4.5 * frequency * damping_ratio;
    trace!("PD gains: kp = {:.3}, kd = {:.3}", kp, kd);

    let mut delta = to_rotation * from_rotation.conjugate();
    if delta.w < 0.0 {
        delta = Quat::from_xyzw(-delta.x, -delta.y, -delta.z, -delta.w);
    }

    let (mut axis, mut angle) = delta.to_axis_angle();
    axis = axis.normalize_or_zero();
    if angle > std::f32::consts::PI {
        angle -= 2.0 * std::f32::consts::PI;
    }

    // Normalize axis (avoid NaNs if angle is zero)
    axis = axis.normalize_or_zero();

    trace!(
        "Rotation error: angle_rad = {:.6} rad (~{:.2}°), axis = {:?}",
        angle,
        angle.to_degrees(),
        axis
    );

    // PD control (raw torque)
    let raw = axis * (kp * angle) - angular_velocity * kd;
    trace!("Raw torque (before inertia scaling) = {:?}", raw);

    let rot_inertia_to_world = inertia_local_frame * from_rotation;
    let torque_local = rot_inertia_to_world.inverse() * raw;
    let torque_scaled = torque_local * inertia_principal;
    let final_torque = rot_inertia_to_world * torque_scaled;

    trace!("Torque after scaling by inertia = {:?}", final_torque);

    // Optionally clamp final torque magnitude
    let torque_to_apply = {
        if final_torque.length_squared() > max_torque * max_torque {
            final_torque.normalize() * max_torque
        } else {
            final_torque
        }
    };

    trace!("Torque to apply (clamped) = {:?}", torque_to_apply);

    torque_to_apply
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_pd_torque_zero_error() {
        let torque = compute_pd_torque(
            1.0,
            1.0,
            10.0,
            Quat::IDENTITY,
            Quat::IDENTITY,
            Vec3::ZERO,
            Vec3::ONE,
            Quat::IDENTITY,
        );
        assert!(torque.abs_diff_eq(Vec3::ZERO, 1e-6));
    }

    #[test]
    fn test_compute_pd_torque_small_angle() {
        let torque = compute_pd_torque(
            1.0,
            1.0,
            10.0,
            Quat::IDENTITY,
            Quat::from_axis_angle(Vec3::Y, 0.1),
            Vec3::ZERO,
            Vec3::ONE,
            Quat::IDENTITY,
        );
        assert!(torque.length() > 0.0);
    }

    #[test]
    fn test_compute_pd_torque_large_angle() {
        let torque = compute_pd_torque(
            1.0,
            1.0,
            10.0,
            Quat::IDENTITY,
            Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI),
            Vec3::ZERO,
            Vec3::ONE,
            Quat::IDENTITY,
        );
        assert!(torque.length() > 0.0);
    }

    #[test]
    fn test_compute_pd_torque_with_angular_velocity() {
        let torque = compute_pd_torque(
            1.0,
            1.0,
            10.0,
            Quat::IDENTITY,
            Quat::from_axis_angle(Vec3::Y, 0.5),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::ONE,
            Quat::IDENTITY,
        );
        assert!(torque.length() > 0.0);
    }
}
