//! Module for defining hull sections in a 3D environment using Bevy and Avian3D.

use avian3d::prelude::*;
use bevy::prelude::*;

pub mod prelude {
    pub use super::hull_section;
    pub use super::HullSectionConfig;
    pub use super::HullSectionMarker;
    pub use super::HullSectionPlugin;
}

const HULL_SECTION_DEFAULT_COLLIDER_DENSITY: f32 = 1.0;

/// Configuration for a hull section.
#[derive(Clone, Debug)]
pub struct HullSectionConfig {
    /// The transform of the hull section relative to its parent.
    pub transform: Transform,
    /// The collider density / mass of the hull section.
    pub collider_density: f32,
    /// The render mesh of the hull section, defaults to a cuboid of size 1x1x1.
    pub render_mesh: Option<Handle<Scene>>,
}

impl Default for HullSectionConfig {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            collider_density: HULL_SECTION_DEFAULT_COLLIDER_DENSITY,
            render_mesh: None,
        }
    }
}

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct HullSectionRenderMesh(Option<Handle<Scene>>);

/// Helper function to create a hull section entity bundle.
pub fn hull_section(config: HullSectionConfig) -> impl Bundle {
    debug!("Creating hull section with config: {:?}", config);

    (
        Name::new("Hull Section"),
        super::SectionMarker,
        HullSectionMarker,
        Collider::cuboid(1.0, 1.0, 1.0),
        ColliderDensity(config.collider_density),
        config.transform,
        Visibility::Visible,
        HullSectionRenderMesh(config.render_mesh),
    )
}

/// Marker component for hull sections.
#[derive(Component, Clone, Debug, Reflect)]
pub struct HullSectionMarker;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HullSectionPluginSet;

/// A plugin that enables the HullSection component and its related systems.
#[derive(Default)]
pub struct HullSectionPlugin {
    pub render: bool,
}

impl Plugin for HullSectionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HullSectionMarker>();

        if self.render {
            app.add_observer(insert_hull_section_render);
        }
    }
}

fn insert_hull_section_render(
    add: On<Add, HullSectionMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_hull: Query<&HullSectionRenderMesh, With<HullSectionMarker>>,
) {
    let entity = add.entity;
    debug!("Inserting render for HullSection: {:?}", entity);
    let Ok(render_mesh) = q_hull.get(entity) else {
        warn!(
            "HullSection entity {:?} missing HullRenderMesh component",
            entity
        );
        return;
    };

    match &**render_mesh {
        Some(scene) => {
            commands.entity(entity).insert((children![(
                Name::new("Hull Section Body"),
                SceneRoot(scene.clone()),
            ),],));
        }
        None => {
            commands.entity(entity).insert((children![(
                Name::new("Hull Section Body"),
                Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                MeshMaterial3d(materials.add(Color::srgb(0.8, 0.8, 0.8))),
            ),],));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hull_section_config_default() {
        let config = HullSectionConfig::default();
        assert_eq!(config.transform, Transform::default());
        assert_eq!(config.collider_density, 1.0);
    }

    #[test]
    fn spawns_hull_with_default_config() {
        // Arrange
        let mut app = App::new();
        let id = app
            .world_mut()
            .spawn(hull_section(HullSectionConfig::default()))
            .id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<HullSectionMarker>(id).is_some());
        assert!(
            **app.world().get::<ColliderDensity>(id).unwrap()
                == HULL_SECTION_DEFAULT_COLLIDER_DENSITY
        );
    }

    #[test]
    fn spawns_hull_with_custom_scene() {
        // Arrange
        let mut app = App::new();
        let custom_scene = Handle::<Scene>::default();
        let config = HullSectionConfig {
            render_mesh: Some(custom_scene.clone()),
            ..Default::default()
        };
        let id = app.world_mut().spawn(hull_section(config)).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<HullSectionMarker>(id).is_some());
        let render_mesh = app.world().get::<HullSectionRenderMesh>(id).unwrap();
        assert!(render_mesh.0.is_some());
        assert_eq!(render_mesh.0.as_ref().unwrap(), &custom_scene);
    }
}
