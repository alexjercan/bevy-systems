//! Defines a thruster section for a spaceship, which provides thrust in a specified direction.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::SpaceshipRootMarker;

pub mod prelude {
    pub use super::thruster_section;
    pub use super::ThrusterSectionConfig;
    pub use super::ThrusterSectionMarker;
    pub use super::ThrusterSectionPlugin;
    pub use super::ThrusterSectionInput;
    pub use super::ThrusterSectionMagnitude;
}

/// Configuration for a thruster section of a spaceship.
#[derive(Default, Clone, Debug)]
pub struct ThrusterSectionConfig {
    /// The magnitude of the force produced by this thruster section.
    pub magnitude: f32,
    /// The transform of the thruster section relative to its parent. This defines the position and
    /// orientation of the thruster section, which in turn defines the direction of the thrust.
    pub transform: Transform,
}

/// Helper function to create an thruster section entity bundle.
pub fn thruster_section(config: ThrusterSectionConfig) -> impl Bundle {
    debug!("Creating thruster section with config: {:?}", config);

    (
        Name::new("Thruster Section"),
        ThrusterSectionMarker,
        Collider::cuboid(1.0, 1.0, 1.0),
        ColliderDensity(1.0),
        ThrusterSectionMagnitude(config.magnitude),
        ThrusterSectionInput(0.0),
        config.transform,
        Visibility::Visible,
    )
}

/// Marker component for thruster sections.
#[derive(Component, Clone, Debug, Reflect)]
pub struct ThrusterSectionMarker;

/// The thrust magnitude produced by this thruster section. This is a simple scalar value that can be
/// used to determine the thrust force applied to the ship.
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct ThrusterSectionMagnitude(pub f32);

/// The thuster input. Will be a value between 0.0 and 1.0, where 0.0 means no thrust and 1.0 means
/// full thrust.
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct ThrusterSectionInput(pub f32);

/// A system set that will contain all the systems related to the thruster section plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThrusterSectionPluginSet;

/// A plugin that enables the ThrusterSection component and its related systems.
pub struct ThrusterSectionPlugin;

impl Plugin for ThrusterSectionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ThrusterSectionMarker>()
            .register_type::<ThrusterSectionMagnitude>()
            .register_type::<ThrusterSectionInput>();
        // TODO: Might add a flag for this later
        app.add_observer(insert_thruster_section_render);

        app.add_systems(FixedUpdate, thruster_impulse_system.in_set(ThrusterSectionPluginSet));
    }
}

fn thruster_impulse_system(
    q_thruster: Query<
        (
            &GlobalTransform,
            &ChildOf,
            &ThrusterSectionMagnitude,
            &ThrusterSectionInput,
        ),
        With<ThrusterSectionMarker>,
    >,
    mut q_root: Query<Forces, With<SpaceshipRootMarker>>,
) {
    for (transform, &ChildOf(root), magnitude, input) in &q_thruster {
        let Ok(mut force) = q_root.get_mut(root) else {
            warn!("ThrusterSection's root entity does not have a RootSectionMarker component");
            continue;
        };

        let thrust_direction = transform.forward(); // Local -Z axis
        let thrust_force = thrust_direction * **magnitude * **input;
        let world_point = transform.translation();

        force.apply_linear_impulse_at_point(thrust_force, world_point);
    }
}

fn insert_thruster_section_render(
    add: On<Add, ThrusterSectionMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = add.entity;
    debug!("Inserting render for ThrusterSection: {:?}", entity);

    commands.entity(entity).insert((children![
        (
            Name::new("Thruster Section Body"),
            Mesh3d(meshes.add(Cylinder::new(0.4, 0.4))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.8, 0.8))),
            Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                .with_translation(Vec3::new(0.0, 0.0, -0.3)),
        ),
        (
            Name::new("Thruster Section Body"),
            Mesh3d(meshes.add(Cone::new(0.5, 0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.3, 0.2))),
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ),
    ],));
}
