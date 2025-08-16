use bevy::prelude::*;

#[cfg(feature = "debug")]
use self::debug::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderPluginSet;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.add_plugins(DebugPlugin);
        #[cfg(feature = "debug")]
        app.configure_sets(Update, DebugPluginSet.in_set(RenderPluginSet));
    }
}

mod debug {
    // TODO: Add a keybind to toggle the unit render system

    use crate::unit::components::*;
    use bevy::prelude::*;

    #[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
    pub struct DebugPluginSet;

    pub struct DebugPlugin;

    impl Plugin for DebugPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Update, handle_unit_render.in_set(DebugPluginSet));
        }
    }

    #[derive(Component)]
    struct UnitRenderReady;

    fn handle_unit_render(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        q_unit: Query<Entity, (With<UnitCoord>, Without<UnitRenderReady>)>,
    ) {
        for entity in q_unit.iter() {
            commands
                .entity(entity)
                .insert(UnitRenderReady)
                .with_children(|parent| {
                    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
                    let material = materials.add(Color::srgb_u8(124, 144, 255));

                    parent.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        Transform::IDENTITY,
                        Visibility::default(),
                        Name::new("Unit Render"),
                    ));
                });
        }
    }
}
