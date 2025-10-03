//! A module for defining engine sections that provide thrust to a ship.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::SpaceshipRootMarker;

pub mod prelude {
    pub use super::engine_section;
    pub use super::EngineSectionConfig;
    pub use super::EngineSectionMarker;
    pub use super::EngineSectionPlugin;
    pub use super::EngineThrustInput;
    pub use super::EngineThrustMagnitude;
}

#[derive(Default, Clone, Debug)]
pub struct EngineSectionConfig {
    /// The magnitude of the thrust produced by this engine section.
    pub thrust_magnitude: f32,
    /// The transform of the engine section relative to its parent. This defines the position and
    /// orientation of the engine section, which in turn defines the direction of the thrust.
    pub transform: Transform,
}

pub fn engine_section(config: EngineSectionConfig) -> impl Bundle {
    (
        Name::new("Engine Section"),
        EngineSectionMarker,
        Collider::cuboid(1.0, 1.0, 1.0),
        ColliderDensity(1.0),
        EngineThrustMagnitude(config.thrust_magnitude),
        EngineThrustInput(0.0),
        config.transform,
        Visibility::Visible,
    )
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct EngineSectionMarker;

/// The thrust magnitude produced by this engine section. This is a simple scalar value that can be
/// used to determine the thrust force applied to the ship. The direction of the thrust is assumed
/// to be along the local -Z axis of the engine section.
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct EngineThrustMagnitude(pub f32);

/// The thuster input. Will be a value between 0.0 and 1.0, where 0.0 means no thrust and 1.0 means
/// full thrust.
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct EngineThrustInput(pub f32);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EngineSectionPluginSet;

pub struct EngineSectionPlugin;

impl Plugin for EngineSectionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EngineSectionMarker>()
            .register_type::<EngineThrustMagnitude>()
            .register_type::<EngineThrustInput>();
        // TODO: Might add a flag for this later
        app.add_observer(insert_engine_section_render);

        app.add_systems(Update, engine_thrust_system.in_set(EngineSectionPluginSet));
    }
}

fn engine_thrust_system(
    q_engines: Query<
        (
            &GlobalTransform,
            &ChildOf,
            &EngineThrustMagnitude,
            &EngineThrustInput,
        ),
        With<EngineSectionMarker>,
    >,
    mut q_root: Query<&mut ExternalImpulse, With<SpaceshipRootMarker>>,
) {
    for (transform, &ChildOf(root), magnitude, input) in &q_engines {
        let Ok(mut force) = q_root.get_mut(root) else {
            warn!("EngineSection's root entity does not have a RootSectionMarker component");
            continue;
        };

        let thrust_direction = transform.forward(); // Local -Z axis
        let thrust_force = thrust_direction * **magnitude * **input;

        force.apply_impulse(thrust_force);
    }
}

fn insert_engine_section_render(
    trigger: Trigger<OnAdd, EngineSectionMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = trigger.target();
    debug!("Inserting render for EngineSection: {:?}", entity);

    commands.entity(entity).insert((children![
        (
            Name::new("Engine Section Body"),
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
