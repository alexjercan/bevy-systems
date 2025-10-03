//! Module for defining hull sections in a 3D environment using Bevy and Avian3D.

use avian3d::prelude::*;
use bevy::prelude::*;

pub mod prelude {
    pub use super::hull_section;
    pub use super::HullSectionConfig;
    pub use super::HullSectionMarker;
    pub use super::HullSectionPlugin;
}

/// Configuration for a hull section.
#[derive(Default, Clone, Debug)]
pub struct HullSectionConfig {
    /// The transform of the hull section relative to its parent.
    pub transform: Transform,
}

/// Helper function to create a hull section entity bundle.
pub fn hull_section(config: HullSectionConfig) -> impl Bundle {
    debug!("Creating hull section with config: {:?}", config);

    (
        Name::new("Hull Section"),
        HullSectionMarker,
        Collider::cuboid(1.0, 1.0, 1.0),
        ColliderDensity(1.0),
        config.transform,
        Visibility::Visible,
    )
}

/// Marker component for hull sections.
#[derive(Component, Clone, Debug, Reflect)]
pub struct HullSectionMarker;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HullSectionPluginSet;

/// A plugin that enables the HullSection component and its related systems.
pub struct HullSectionPlugin;

impl Plugin for HullSectionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HullSectionMarker>();
        // TODO: Might add a flag for this later
        app.add_observer(insert_hull_section_render);
    }
}

fn insert_hull_section_render(
    trigger: Trigger<OnAdd, HullSectionMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = trigger.target();
    debug!("Inserting render for HullSection: {:?}", entity);

    commands.entity(entity).insert((
        children![(
            Name::new("Hull Section Body"),
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.8, 0.8))),
        ),],
    ));
}
