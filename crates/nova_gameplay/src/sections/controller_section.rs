//! A section of a spaceship that can control its rotation using a PD controller.

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::prelude::SpaceshipRootMarker;

pub mod prelude {
    pub use super::controller_section;
    pub use super::ControllerSectionConfig;
    pub use super::ControllerSectionMarker;
    pub use super::ControllerSectionPlugin;
    pub use super::ControllerSectionRotationInput;
    pub use super::ControllerSectionStableTorquePdController;
}

const CONTROLLER_SECTION_DEFAULT_COLLIDER_DENSITY: f32 = 1.0;

/// Configuration for a controller section.
#[derive(Clone, Debug)]
pub struct ControllerSectionConfig {
    /// The transform of the controller section relative to its parent.
    pub transform: Transform,
    /// The frequency of the PD controller in Hz.
    pub frequency: f32,
    /// The damping ratio of the PD controller.
    pub damping_ratio: f32,
    /// The maximum torque that can be applied by the PD controller.
    pub max_torque: f32,
    /// The collider density / mass of the hull section.
    pub collider_density: f32,
    /// The render mesh of the hull section, defaults to a cuboid of size 1x1x1.
    pub render_mesh: Option<Handle<Scene>>,
}

impl Default for ControllerSectionConfig {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            frequency: 2.0,
            damping_ratio: 2.0,
            max_torque: 1.0,
            collider_density: CONTROLLER_SECTION_DEFAULT_COLLIDER_DENSITY,
            render_mesh: None,
        }
    }
}

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct ControllerSectionRenderMesh(Option<Handle<Scene>>);

/// Helper function to create a controller section entity bundle.
pub fn controller_section(config: ControllerSectionConfig) -> impl Bundle {
    debug!("Creating controller section with config: {:?}", config);

    (
        Name::new("Controller Section"),
        super::SectionMarker,
        ControllerSectionMarker,
        Collider::cuboid(1.0, 1.0, 1.0),
        ColliderDensity(config.collider_density),
        ControllerSectionRotationInput::default(),
        ControllerSectionStableTorquePdController {
            frequency: config.frequency,
            damping_ratio: config.damping_ratio,
            max_torque: config.max_torque,
        },
        config.transform,
        Visibility::Visible,
        ControllerSectionRenderMesh(config.render_mesh),
    )
}

/// Marker component for controller sections.
#[derive(Component, Clone, Debug, Reflect)]
pub struct ControllerSectionMarker;

/// The desired rotation of the controller section, in world space.
#[derive(Component, Debug, Clone, Default, Deref, DerefMut, Reflect)]
pub struct ControllerSectionRotationInput(pub Quat);

/// A stable PD controller that applies torque to maintain a desired rotation.
#[derive(Component, Debug, Clone, Reflect)]
pub struct ControllerSectionStableTorquePdController {
    /// Frequency in Hz.
    pub frequency: f32,
    /// Damping ratio.
    pub damping_ratio: f32,
    /// Maximum torque that can be applied.
    pub max_torque: f32,
}

/// A system set that will contain all the systems related to the controller section plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ControllerSectionPluginSet;

/// A plugin that will enable the ControllerSection.
#[derive(Default)]
pub struct ControllerSectionPlugin {
    pub render: bool,
}

impl Plugin for ControllerSectionPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: How can we check that the TorquePdControllerPlugin is added?
        app.register_type::<ControllerSectionMarker>();

        app.add_systems(
            FixedUpdate,
            update_controller_root_torque.in_set(ControllerSectionPluginSet),
        );

        if self.render {
            app.add_observer(insert_controller_section_render);
        }
    }
}

fn update_controller_root_torque(
    mut q_root: Query<(&ComputedAngularInertia, &Rotation, Forces), With<SpaceshipRootMarker>>,
    q_controller: Query<
        (
            &ControllerSectionStableTorquePdController,
            &ControllerSectionRotationInput,
            &ChildOf,
        ),
        With<ControllerSectionMarker>,
    >,
) {
    for (controller, controller_input, &ChildOf(root)) in &q_controller {
        let Ok((angular_inertia, rotation, mut forces)) = q_root.get_mut(root) else {
            warn!("ControllerSection's root entity does not have a SpaceshipRootMaker component");
            continue;
        };

        let (principal, local_frame) = angular_inertia.principal_angular_inertia_with_local_frame();

        let torque = compute_pd_torque(
            controller.frequency,
            controller.damping_ratio,
            controller.max_torque,
            **rotation,
            **controller_input,
            forces.angular_velocity(),
            principal,
            local_frame,
        );

        forces.apply_torque(torque);
    }
}

fn insert_controller_section_render(
    add: On<Add, ControllerSectionMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_controller: Query<&ControllerSectionRenderMesh, With<ControllerSectionMarker>>,
) {
    let entity = add.entity;
    debug!("Inserting render for ControllerSection: {:?}", entity);

    let Ok(render_mesh) = q_controller.get(entity) else {
        warn!(
            "ControllerSection entity {:?} missing ControllerSectionRenderMesh component",
            entity
        );
        return;
    };

    match &**render_mesh {
        Some(scene) => {
            commands.entity(entity).insert((children![(
                Name::new("Controller Section Body"),
                SceneRoot(scene.clone()),
            ),],));
        }
        None => {
            commands.entity(entity).insert((children![
                (
                    Name::new("Controller Section Body (A)"),
                    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                    MeshMaterial3d(materials.add(Color::srgb(0.2, 0.7, 0.9))),
                ),
                (
                    Name::new("Controller Section Window (B)"),
                    Mesh3d(meshes.add(Cylinder::new(0.2, 0.1))),
                    MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 1.0))),
                    Transform::from_xyz(0.0, 0.5, 0.0),
                )
            ],));
        }
    }
}

fn compute_pd_torque(
    frequency: f32,
    damping_ratio: f32,
    max_torque: f32,
    from_rotation: Quat,
    to_rotation: Quat,
    angular_velocity: Vec3,
    inertia_principal: Vec3,
    inertia_local_frame: Quat,
) -> Vec3 {
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

    #[test]
    fn spawns_controller_with_default_config() {
        // Arrange
        let mut app = App::new();
        let id = app
            .world_mut()
            .spawn(controller_section(ControllerSectionConfig::default()))
            .id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<ControllerSectionMarker>(id).is_some());
        assert!(
            **app.world().get::<ColliderDensity>(id).unwrap()
                == CONTROLLER_SECTION_DEFAULT_COLLIDER_DENSITY
        );
    }

    #[test]
    fn spawns_controller_with_custom_scene() {
        // Arrange
        let mut app = App::new();
        let custom_scene = Handle::<Scene>::default();
        let config = ControllerSectionConfig {
            render_mesh: Some(custom_scene.clone()),
            ..Default::default()
        };
        let id = app.world_mut().spawn(controller_section(config)).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<ControllerSectionMarker>(id).is_some());
        let render_mesh = app.world().get::<ControllerSectionRenderMesh>(id).unwrap();
        assert!(render_mesh.0.is_some());
        assert_eq!(render_mesh.0.as_ref().unwrap(), &custom_scene);
    }
}
