//! Defines a thruster section for a spaceship, which provides thrust in a specified direction.

use avian3d::prelude::*;
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

use crate::prelude::{SpaceshipRootMarker, SpaceshipSystems};

pub mod prelude {
    pub use super::thruster_section;
    pub use super::ThrusterSectionConfig;
    pub use super::ThrusterSectionInput;
    pub use super::ThrusterSectionMagnitude;
    pub use super::ThrusterSectionMarker;
    pub use super::ThrusterSectionPlugin;
}

const THRUSTER_SECTION_DEFAULT_MAGNITUDE: f32 = 1.0;

/// Configuration for a thruster section of a spaceship.
#[derive(Clone, Debug)]
pub struct ThrusterSectionConfig {
    /// The magnitude of the force produced by this thruster section.
    pub magnitude: f32,
    /// The render mesh of the section, defaults to prototype mesh if None.
    pub render_mesh: Option<Handle<Scene>>,
}

impl Default for ThrusterSectionConfig {
    fn default() -> Self {
        Self {
            magnitude: THRUSTER_SECTION_DEFAULT_MAGNITUDE,
            render_mesh: None,
        }
    }
}

/// Helper function to create an thruster section entity bundle.
pub fn thruster_section(config: ThrusterSectionConfig) -> impl Bundle {
    debug!("Creating thruster section with config: {:?}", config);

    (
        ThrusterSectionMarker,
        ThrusterSectionMagnitude(config.magnitude),
        ThrusterSectionInput(0.0),
        ThrusterSectionRenderMesh(config.render_mesh),
    )
}

/// Marker component for thruster sections.
#[derive(Component, Clone, Debug, Reflect)]
pub struct ThrusterSectionMarker;

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct ThrusterSectionRenderMesh(Option<Handle<Scene>>);

/// The thrust magnitude produced by this thruster section. This is a simple scalar value that can be
/// used to determine the thrust force applied to the ship.
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct ThrusterSectionMagnitude(pub f32);

/// The thuster input. Will be a value between 0.0 and 1.0, where 0.0 means no thrust and 1.0 means
/// full thrust.
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct ThrusterSectionInput(pub f32);

/// A plugin that enables the ThrusterSection component and its related systems.
#[derive(Default)]
pub struct ThrusterSectionPlugin {
    pub render: bool,
}

impl Plugin for ThrusterSectionPlugin {
    fn build(&self, app: &mut App) {
        debug!("ThrusterSectionPlugin: build");

        if self.render {
            app.add_plugins(MaterialPlugin::<
                ExtendedMaterial<StandardMaterial, ThrusterExhaustMaterial>,
            >::default());

            app.add_observer(insert_thruster_section_render);
        }

        app.add_systems(
            Update,
            thruster_shader_update_system.in_set(SpaceshipSystems::Sections),
        );
        app.add_systems(
            FixedUpdate,
            thruster_impulse_system.in_set(SpaceshipSystems::Sections),
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
            warn!(
                "thruster_impulse_system: entity {:?} not found in q_root",
                root
            );
            continue;
        };

        let thrust_direction = transform.forward(); // Local -Z axis
        let thrust_force = thrust_direction * **magnitude * **input;
        let world_point = transform.translation();

        force.apply_linear_impulse_at_point(thrust_force, world_point);
    }
}

#[derive(Component, Clone, Debug, Reflect)]
struct ThrusterSectionExhaustShaderMarker;

fn thruster_shader_update_system(
    q_thruster: Query<&ThrusterSectionInput, With<ThrusterSectionMarker>>,
    q_render: Query<
        (
            &MeshMaterial3d<ExtendedMaterial<StandardMaterial, ThrusterExhaustMaterial>>,
            &ChildOf,
        ),
        With<ThrusterSectionExhaustShaderMarker>,
    >,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ThrusterExhaustMaterial>>>,
) {
    for (material, &ChildOf(parent)) in &q_render {
        let Ok(input) = q_thruster.get(parent) else {
            warn!(
                "thruster_shader_update_system: entity {:?} not found in q_thruster",
                parent
            );
            continue;
        };

        let Some(material) = materials.get_mut(&**material) else {
            warn!(
                "thruster_shader_update_system: material for entity {:?} not found",
                parent
            );
            continue;
        };

        material.extension.thruster_input = **input;
    }
}

fn insert_thruster_section_render(
    add: On<Add, ThrusterSectionMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut exhaust_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, ThrusterExhaustMaterial>>,
    >,
    q_thruster: Query<&ThrusterSectionRenderMesh, With<ThrusterSectionMarker>>,
) {
    let entity = add.entity;
    trace!("insert_thruster_section_render: entity {:?}", entity);

    let Ok(render_mesh) = q_thruster.get(entity) else {
        warn!(
            "insert_thruster_section_render: entity {:?} not found in q_thruster",
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
                    MeshMaterial3d(standard_materials.add(Color::srgb(0.8, 0.8, 0.8))),
                    Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                        .with_translation(Vec3::new(0.0, 0.0, -0.3)),
                ),
                (
                    Name::new("Thruster Section Body (B)"),
                    Mesh3d(meshes.add(Cone::new(0.5, 0.5))),
                    MeshMaterial3d(standard_materials.add(Color::srgb(0.9, 0.3, 0.2))),
                    Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                ),
                (
                    Name::new("Thruster Exhaust"),
                    ThrusterSectionExhaustShaderMarker,
                    Mesh3d(meshes.add(Cone::new(0.4, 0.1))),
                    MeshMaterial3d(
                        exhaust_materials.add(ExtendedMaterial {
                            base: StandardMaterial {
                                base_color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                                perceptual_roughness: 1.0,
                                metallic: 0.0,
                                emissive: LinearRgba::rgb(0.0, 10.0, 10.0),
                                ..default()
                            },
                            extension: ThrusterExhaustMaterial::default()
                                .with_exhaust_height(1.0)
                                .with_exhaust_radius(0.4),
                        })
                    ),
                    Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                        .with_translation(Vec3::new(0.0, 0.0, 0.3)),
                ),
            ],));
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct ThrusterExhaustMaterial {
    #[uniform(100)]
    pub thruster_input: f32,
    #[uniform(100)]
    pub thruster_exhaust_radius: f32,
    #[uniform(100)]
    pub thruster_exhaust_height: f32,
    #[cfg(target_arch = "wasm32")]
    #[uniform(100)]
    _webgl2_padding_16b: u32,
}

impl ThrusterExhaustMaterial {
    pub fn with_exhaust_radius(mut self, radius: f32) -> Self {
        self.thruster_exhaust_radius = radius;
        self
    }

    pub fn with_exhaust_height(mut self, height: f32) -> Self {
        self.thruster_exhaust_height = height;
        self
    }
}

impl MaterialExtension for ThrusterExhaustMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/thruster_exhaust.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/thruster_exhaust.wgsl".into()
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
