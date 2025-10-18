/// A Bevy plugin that makes entities explode into pieces when they are destroyed.

use bevy::prelude::*;
use bevy_common_systems::prelude::*;

pub mod prelude {
    pub use super::ExplodeOnDestroyPlugin;
    pub use super::ExplodeOnDestroyPluginSet;
    pub use super::ExplodeOnDestroy;
}

/// A component that makes an entity explode into pieces when it is destroyed.
#[derive(Component, Default, Clone, Debug, Reflect)]
pub struct ExplodeOnDestroy {
    /// The entity that contains the mesh we will use for the "explosion"
    pub mesh_entity: Option<Entity>,
    /// Number of fragments (planes) to split the mesh into
    pub fragment_count: usize,
}

/// A system set that will contain all the systems related to the explode on destroy plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExplodeOnDestroyPluginSet;

/// A plugin that makes entities explode into pieces when they are destroyed.
pub struct ExplodeOnDestroyPlugin;

impl Plugin for ExplodeOnDestroyPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_explosion);
    }
}

fn handle_explosion(
    add: On<Add, DestroyedMarker>,
    mut commands: Commands,
    q_explode: Query<&ExplodeOnDestroy>,
    mut q_mesh: Query<(&Mesh3d, &mut Visibility)>,
    meshes: Res<Assets<Mesh>>,
) {
    let entity = add.entity;
    debug!("Handling explosion for entity {:?}", entity);

    let Ok(explode) = q_explode.get(entity) else {
        warn!("Destroyed entity {:?} missing ExplodeOnDestroy component, skipping explosion.", entity);
        return;
    };

    let Some(mesh_entity) = explode.mesh_entity else {
        debug!("ExplodeOnDestroy component on entity {:?} has no mesh_entity set, skipping explosion.", entity);
        return;
    };

    let Ok((mesh3d, mut visibility)) = q_mesh.get_mut(mesh_entity) else {
        warn!("Mesh entity {:?} for explosion on entity {:?} does not have a Mesh3d component, skipping explosion.", mesh_entity, entity);
        return;
    };

    let Some(mesh) = meshes.get(&**mesh3d) else {
        warn!("Mesh asset for entity {:?} not found, skipping explosion.", mesh_entity);
        return;
    };

    println!(
        "Exploding entity {:?} with mesh entity {:?} into {} fragments.",
        entity, mesh_entity, explode.fragment_count
    );

    *visibility = Visibility::Hidden;
}
