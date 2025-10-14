//! Defines a thruster section for a spaceship, which provides thrust in a specified direction.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::SpaceshipRootMarker;

pub mod prelude {
    pub use super::thruster_section;
    pub use super::ThrusterSectionConfig;
    pub use super::ThrusterSectionInput;
    pub use super::ThrusterSectionMagnitude;
    pub use super::ThrusterSectionMarker;
    pub use super::ThrusterSectionPlugin;
}

const THRUSTER_SECTION_DEFAULT_MAGNITUDE: f32 = 1.0;
const THRUSTER_SECTION_DEFAULT_COLLIDER_DENSITY: f32 = 1.0;

/// Configuration for a thruster section of a spaceship.
#[derive(Clone, Debug)]
pub struct ThrusterSectionConfig {
    /// The magnitude of the force produced by this thruster section.
    pub magnitude: f32,
    /// The transform of the thruster section relative to its parent. This defines the position and
    /// orientation of the thruster section, which in turn defines the direction of the thrust.
    pub transform: Transform,
    /// The collider density / mass of the section.
    pub collider_density: f32,
    /// The render mesh of the section, defaults to prototype mesh if None.
    pub render_mesh: Option<Handle<Scene>>,
}

impl Default for ThrusterSectionConfig {
    fn default() -> Self {
        Self {
            magnitude: THRUSTER_SECTION_DEFAULT_MAGNITUDE,
            transform: Transform::default(),
            collider_density: THRUSTER_SECTION_DEFAULT_COLLIDER_DENSITY,
            render_mesh: None,
        }
    }
}

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct ThrusterSectionRenderMesh(Option<Handle<Scene>>);

/// Helper function to create an thruster section entity bundle.
pub fn thruster_section(config: ThrusterSectionConfig) -> impl Bundle {
    debug!("Creating thruster section with config: {:?}", config);

    (
        Name::new("Thruster Section"),
        ThrusterSectionMarker,
        Collider::cuboid(1.0, 1.0, 1.0),
        ColliderDensity(config.collider_density),
        ThrusterSectionMagnitude(config.magnitude),
        ThrusterSectionInput(0.0),
        config.transform,
        Visibility::Visible,
        ThrusterSectionRenderMesh(config.render_mesh),
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
#[derive(Default)]
pub struct ThrusterSectionPlugin {
    pub render: bool,
}

impl Plugin for ThrusterSectionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ThrusterSectionMarker>()
            .register_type::<ThrusterSectionMagnitude>()
            .register_type::<ThrusterSectionInput>();

        if self.render {
            app.add_observer(insert_thruster_section_render);
        }

        app.add_systems(
            FixedUpdate,
            thruster_impulse_system.in_set(ThrusterSectionPluginSet),
        );
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
    q_thruster: Query<&ThrusterSectionRenderMesh, With<ThrusterSectionMarker>>,
) {
    let entity = add.entity;
    debug!("Inserting render for ThrusterSection: {:?}", entity);
    let Ok(render_mesh) = q_thruster.get(entity) else {
        warn!(
            "ThrusterSection entity {:?} missing ThrusterRenderMesh component",
            entity
        );
        return;
    };

    match &**render_mesh {
        Some(scene) => {
            commands.entity(entity).insert((children![(
                Name::new("Thruster Section Body"),
                SceneRoot(scene.clone()),
            ),],));
        }
        None => {
            commands.entity(entity).insert((children![
                (
                    Name::new("Thruster Section Body (A)"),
                    Mesh3d(meshes.add(Cylinder::new(0.4, 0.4))),
                    MeshMaterial3d(materials.add(Color::srgb(0.8, 0.8, 0.8))),
                    Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                        .with_translation(Vec3::new(0.0, 0.0, -0.3)),
                ),
                (
                    Name::new("Thruster Section Body (B)"),
                    Mesh3d(meshes.add(Cone::new(0.5, 0.5))),
                    MeshMaterial3d(materials.add(Color::srgb(0.9, 0.3, 0.2))),
                    Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                ),
            ],));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn spawns_thruster_with_default_config() {
        // Arrange
        let mut app = App::new();
        let id = app
            .world_mut()
            .spawn(thruster_section(ThrusterSectionConfig::default()))
            .id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<ThrusterSectionMarker>(id).is_some());
        assert!(
            **app.world().get::<ColliderDensity>(id).unwrap()
                == THRUSTER_SECTION_DEFAULT_COLLIDER_DENSITY
        );
    }

    #[test]
    fn spawns_thruster_with_custom_scene() {
        // Arrange
        let mut app = App::new();
        let custom_scene = Handle::<Scene>::default();
        let config = ThrusterSectionConfig {
            render_mesh: Some(custom_scene.clone()),
            ..default()
        };
        let id = app.world_mut().spawn(thruster_section(config)).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<ThrusterSectionMarker>(id).is_some());
        let render_mesh = app.world().get::<ThrusterSectionRenderMesh>(id).unwrap();
        assert!(render_mesh.0.is_some());
        assert_eq!(render_mesh.0.as_ref().unwrap(), &custom_scene);
    }
}
