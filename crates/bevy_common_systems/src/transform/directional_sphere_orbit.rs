use crate::meth::prelude::*;
use bevy::prelude::*;

pub mod prelude {
    pub use super::{
        DirectionalSphereOrbit, DirectionalSphereOrbitInput, DirectionalSphereOrbitOutput,
        DirectionalSphereOrbitPlugin, DirectionalSphereOrbitPluginSet,
    };
}

/// Component to define a spherical orbit around a center point.
#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct DirectionalSphereOrbit {
    /// Radius of the sphere (distance from origin or from a center)
    pub radius: f32,
    /// (Optional) center of the sphere (in world space)
    pub center: Vec3,
    /// Initial pointing direction
    pub direction: Vec3,
    /// Smoothing factor (between 0 and 1) for the orbit movement
    /// 0 = no smoothing, 1 = full smoothing
    pub smoothing: f32,
}

#[derive(Component, Clone, Debug, Default, Deref, DerefMut, Reflect)]
pub struct DirectionalSphereOrbitOutput(pub Vec3);

#[derive(Component, Default, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct DirectionalSphereOrbitInput(pub Vec3);

#[derive(Component, Clone, Debug, Default, Reflect)]
struct DirectionalSphereOrbitState {
    theta: f32,
    phi: f32,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectionalSphereOrbitPluginSet;

/// Plugin to manage entities with `DirectionalSphereOrbit` component.
///
/// DirectionalSphereOrbit allows an entity to orbit around a point on the surface of a sphere.
pub struct DirectionalSphereOrbitPlugin;

impl Plugin for DirectionalSphereOrbitPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(initialize_sphere_orbit_system);

        app.add_systems(
            Update,
            (sphere_update_state, sphere_update_output)
                .chain()
                .in_set(DirectionalSphereOrbitPluginSet),
        );
    }
}

/// Initialize orbit state and next target angles
fn initialize_sphere_orbit_system(
    insert: On<Insert, DirectionalSphereOrbit>,
    mut commands: Commands,
    q_orbit: Query<&DirectionalSphereOrbit, With<DirectionalSphereOrbit>>,
) {
    let entity = insert.entity;
    let Ok(orbit) = q_orbit.get(entity) else {
        warn!(
            "initialize_sphere_orbit_system: entity does not have DirectionalSphereOrbit component"
        );
        return;
    };

    let (theta, phi) = direction_to_spherical(orbit.direction);

    commands.entity(entity).insert((
        DirectionalSphereOrbitState { theta, phi },
        DirectionalSphereOrbitInput(orbit.direction),
        DirectionalSphereOrbitOutput(
            spherical_to_cartesian(orbit.radius, theta, phi) + orbit.center,
        ),
    ));
}

fn sphere_update_state(
    time: Res<Time>,
    mut query: Query<(
        &DirectionalSphereOrbit,
        &mut DirectionalSphereOrbitState,
        &DirectionalSphereOrbitInput,
    )>,
) {
    let dt = time.delta_secs();

    for (orbit, mut state, next) in query.iter_mut() {
        let (new_theta, new_phi) = direction_to_spherical(**next);

        let smoothing = orbit.smoothing.clamp(0.0, 1.0);
        let new_theta = state.theta.lerp_and_snap(new_theta, smoothing, dt);
        let new_phi = state.phi.lerp_and_snap(new_phi, smoothing, dt);

        state.theta = new_theta;
        state.phi = new_phi;
    }
}

fn sphere_update_output(
    mut query: Query<(
        &DirectionalSphereOrbit,
        &DirectionalSphereOrbitState,
        &mut DirectionalSphereOrbitOutput,
    )>,
) {
    for (orbit, state, mut output) in query.iter_mut() {
        let pos = spherical_to_cartesian(orbit.radius, state.theta, state.phi) + orbit.center;
        output.0 = pos;
    }
}
