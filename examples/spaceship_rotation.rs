//! In this example, I want to demo how to use StableTorquePdController to rotate a spaceship to
//! follow the mouse cursor. The spaceship will rotate to face the mouse cursor when moved.

use avian3d::{math::*, prelude::*};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_systems::prelude::*;
use clap::Parser;
use rand::prelude::*;

#[derive(Parser)]
#[command(name = "spaceship_rotation")]
#[command(version = "0.1")]
#[command(about = "Example for the StableTorquePdController", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();

    let mut app = new_gui_app();

    app.add_plugins(PhysicsPlugins::default());
    app.insert_resource(Gravity::ZERO);
    app.add_systems(Startup, setup);

    app.add_plugins(EnhancedInputPlugin);
    app.add_input_context::<CameraInputMarker>();
    app.add_observer(update_camera_rotation_input);
    app.add_observer(update_camera_zoom_input);

    app.add_plugins(OrbitCameraPlugin);
    app.add_plugins(SphereRandomOrbitPlugin);
    app.add_plugins(StableTorquePdControllerPlugin);
    app.add_systems(Update, update_spaceship_target_rotation);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Setting up the scene...");

    // Spawn a 3D camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 20.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        OrbitCamera::default(),
        CameraInputMarker,
        actions!(
            CameraInputMarker[
                (
                    Action::<CameraInputRotate>::new(),
                    Bindings::spawn((
                        // Bevy requires single entities to be wrapped in `Spawn`.
                        // You can attach modifiers to individual bindings as well.
                        Spawn((Binding::mouse_motion(), Scale::splat(0.1), Negate::all())),
                        Axial::right_stick().with((Scale::splat(2.0), Negate::x())),
                    )),
                ),
                (
                    Action::<CameraInputZoom>::new(),
                    Scale::splat(1.0),
                    Bindings::spawn((
                        Spawn((Binding::mouse_wheel(), SwizzleAxis::YXZ)),
                        Bidirectional::up_down_dpad(),
                    ))
                ),
            ]
        ),
    ));

    // Spawn a light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -FRAC_PI_2, 0.0, 0.0)),
        GlobalTransform::default(),
    ));

    // Spawn a spaceship entity (a rectangle with some features to figure out its orientation)
    commands.spawn((
        Name::new("Spaceship"),
        RigidBody::Dynamic,
        Collider::cylinder(0.5, 1.0),
        ColliderDensity(2.0),
        StableTorquePdController {
            frequency: 2.0,
            damping_ratio: 1.0,
            max_torque: 10.0,
        },
        Transform::default(),
        Visibility::Visible,
        children![
            (
                Name::new("Spaceship Renderer"),
                Mesh3d(meshes.add(Cylinder::new(0.5, 1.0))),
                MeshMaterial3d(materials.add(Color::srgb(0.2, 0.7, 0.9))),
                Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
            ),
            (
                Name::new("Spaceship Thruster"),
                Mesh3d(meshes.add(Cone::new(0.5, 0.5))),
                MeshMaterial3d(materials.add(Color::srgb(0.9, 0.3, 0.2))),
                Transform::from_xyz(0.0, 0.0, 0.5).with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
            )
        ],
    ));

    // Spawn a target entity to visualize the target rotation
    commands.spawn((
        Name::new("Spaceship Rotation Target"),
        SpaceshipRotationTargetMarker,
        SphereOrbit {
            radius: 5.0,
            angular_speed: 5.0,
            center: Vec3::ZERO,
        },
        StableTorquePdControllerTarget(Quat::IDENTITY),
        Transform::from_xyz(0.0, 0.0, -5.0),
        Visibility::Visible,
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.2))),
    ));
}

#[derive(Component, Debug, Clone)]
struct CameraInputMarker;

#[derive(InputAction)]
#[action_output(Vec2)]
struct CameraInputRotate;

#[derive(InputAction)]
#[action_output(f32)]
struct CameraInputZoom;

fn update_camera_rotation_input(
    trigger: Trigger<Fired<CameraInputRotate>>,
    mut q_input: Query<&mut OrbitCameraInput, With<CameraInputMarker>>,
) {
    if let Ok(mut input) = q_input.get_mut(trigger.target()) {
        input.orbit = trigger.value;
    }
}

fn update_camera_zoom_input(
    trigger: Trigger<Fired<CameraInputZoom>>,
    mut q_input: Query<&mut OrbitCameraInput, With<CameraInputMarker>>,
) {
    if let Ok(mut input) = q_input.get_mut(trigger.target()) {
        input.zoom = trigger.value;
    }
}

#[derive(Component, Debug, Clone)]
struct SpaceshipRotationTargetMarker;

fn update_spaceship_target_rotation(
    target: Single<&Transform, With<SpaceshipRotationTargetMarker>>,
    controller: Single<
        (&mut StableTorquePdControllerTarget, &Transform),
        With<StableTorquePdController>,
    >,
) {
    let target_transform = target.into_inner();
    let (mut controller_target, controller_transform) = controller.into_inner();

    let direction =
        (target_transform.translation - controller_transform.translation).normalize_or_zero();
    let forward = controller_transform.forward();
    let angle = forward.angle_between(direction);
    let axis = forward.cross(direction).normalize_or_zero();

    let target_rotation = Quat::from_axis_angle(axis, angle) * controller_transform.rotation;

    **controller_target = target_rotation;
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
    debug!("PD gains: kp = {:.3}, kd = {:.3}", kp, kd);

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

    debug!(
        "Rotation error: angle_rad = {:.6} rad (~{:.2}°), axis = {:?}",
        angle,
        angle.to_degrees(),
        axis
    );

    // PD control (raw torque)
    let raw = axis * (kp * angle) - angular_velocity * kd;
    debug!("Raw torque (before inertia scaling) = {:?}", raw);

    let rot_inertia_to_world = inertia_local_frame * from_rotation;
    let torque_local = rot_inertia_to_world.inverse() * raw;
    let torque_scaled = torque_local * inertia_principal;
    let final_torque = rot_inertia_to_world * torque_scaled;

    debug!("Torque after scaling by inertia = {:?}", final_torque);

    // Optionally clamp final torque magnitude
    let torque_to_apply = {
        if final_torque.length_squared() > max_torque * max_torque {
            final_torque.normalize() * max_torque
        } else {
            final_torque
        }
    };

    debug!("Torque to apply (clamped) = {:?}", torque_to_apply);

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

#[derive(Component, Clone, Debug, Reflect)]
#[require(SphereOrbitState, Transform)]
/// Component to define a spherical orbit around a center point.
pub struct SphereOrbit {
    /// Radius of the sphere (distance from origin or from a center)
    pub radius: f32,
    /// Speed (in radians per second) of movement along the sphere surface
    pub angular_speed: f32,
    /// (Optional) center of the sphere (in world space)
    pub center: Vec3,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[require(SphereOrbitNext)]
struct SphereOrbitState {
    theta: f32,
    phi: f32,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
struct SphereOrbitNext {
    theta: f32,
    phi: f32,
}

pub struct SphereRandomOrbitPlugin;

impl Plugin for SphereRandomOrbitPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SphereOrbit>()
            .register_type::<SphereOrbitState>()
            .register_type::<SphereOrbitNext>();

        app.add_systems(
            Update,
            (
                sphere_random_orbit_next_system,
                sphere_random_orbit_follow_system,
            ),
        );
    }
}

/// System: pick a new random “next” angles for orbits that have reached (or nearly reached) their current next
fn sphere_random_orbit_next_system(mut query: Query<(&SphereOrbitState, &mut SphereOrbitNext)>) {
    let mut rng = rand::rng();

    for (state, mut next) in query.iter_mut() {
        let dtheta = (next.theta - state.theta).abs();
        let dphi = (next.phi - state.phi).abs();

        let threshold = 0.01;
        if dtheta < threshold && dphi < threshold {
            let new_theta = rng.random_range(0.0..(std::f32::consts::TAU));

            let new_phi =
                rng.random_range(-std::f32::consts::FRAC_PI_2..std::f32::consts::FRAC_PI_2);
            next.theta = new_theta;
            next.phi = new_phi;
        }
    }
}

/// System: move the state toward `next` gradually, and update the Transform
fn sphere_random_orbit_follow_system(
    time: Res<Time>,
    mut query: Query<(
        &SphereOrbit,
        &mut SphereOrbitState,
        &SphereOrbitNext,
        &mut Transform,
    )>,
) {
    let dt = time.delta_secs();

    for (orbit, mut state, next, mut tf) in query.iter_mut() {
        // Interpolate angles toward next
        let delta_theta = next.theta - state.theta;
        let delta_phi = next.phi - state.phi;

        // We can move with angular_speed; i.e. maximum angular change per second
        let max_delta = orbit.angular_speed * dt;

        // Move theta
        let new_theta = if delta_theta.abs() <= max_delta {
            next.theta
        } else {
            state.theta + delta_theta.signum() * max_delta
        };

        // Move phi
        let new_phi = if delta_phi.abs() <= max_delta {
            next.phi
        } else {
            state.phi + delta_phi.signum() * max_delta
        };

        state.theta = new_theta;
        state.phi = new_phi;

        // Convert spherical to Cartesian
        // theta: azimuth around Y axis; phi: elevation from equator
        let cos_phi = state.phi.cos();
        let x = orbit.radius * cos_phi * state.theta.cos();
        let y = orbit.radius * state.phi;
        let z = orbit.radius * cos_phi * state.theta.sin();

        let new_pos = orbit.center + Vec3::new(x, y, z);
        tf.translation = new_pos;
    }
}
