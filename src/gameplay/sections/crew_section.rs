use avian3d::prelude::*;
use bevy::prelude::*;

pub mod prelude {
    pub use super::crew_section;
    pub use super::CrewSectionConfig;
    pub use super::CrewSectionMarker;
    pub use super::CrewSectionPlugin;
}

#[derive(Default, Clone, Debug)]
pub struct CrewSectionConfig {
    pub transform: Transform,
}

pub fn crew_section(config: CrewSectionConfig) -> impl Bundle {
    (
        Name::new("Crew Section"),
        CrewSectionMarker,
        Collider::cuboid(1.0, 1.0, 1.0),
        ColliderDensity(1.0),
        config.transform,
        Visibility::Visible,
    )
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct CrewSectionMarker;

pub struct CrewSectionPlugin;

impl Plugin for CrewSectionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CrewSectionMarker>();
        // TODO: Might add a flag for this later
        app.add_observer(insert_crew_section_render);
    }
}

fn insert_crew_section_render(
    trigger: Trigger<OnAdd, CrewSectionMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = trigger.target();
    debug!("Inserting render for CrewSection: {:?}", entity);

    commands.entity(entity).insert((
        children![
            (
                Name::new("Crew Section Body"),
                Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                MeshMaterial3d(materials.add(Color::srgb(0.2, 0.7, 0.9))),
            ),
            (
                Name::new("Crew Section Window"),
                Mesh3d(meshes.add(Cylinder::new(0.2, 0.1))),
                MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 1.0))),
                Transform::from_xyz(0.0, 0.5, 0.0),
            )
        ],
    ));
}
