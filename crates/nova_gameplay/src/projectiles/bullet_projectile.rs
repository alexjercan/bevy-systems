use bevy::prelude::*;
use bevy_common_systems::prelude::*;

pub mod prelude {
    pub use super::bullet_projectile;
    pub use super::BulletProjectileConfig;
    pub use super::BulletProjectileMarker;
    pub use super::BulletProjectilePlugin;
    pub use super::BulletProjectileRenderMesh;
}

#[derive(Clone, Debug)]
pub struct BulletProjectileConfig {
    pub lifetime: f32,
    pub render_mesh: Option<Handle<Scene>>,
}

impl Default for BulletProjectileConfig {
    fn default() -> Self {
        Self {
            lifetime: 5.0,
            render_mesh: None,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct BulletProjectileMarker;

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct BulletProjectileRenderMesh(pub Option<Handle<Scene>>);

pub fn bullet_projectile(config: BulletProjectileConfig) -> impl Bundle {
    (
        BulletProjectileMarker,
        TempEntity(config.lifetime),
        BulletProjectileRenderMesh(config.render_mesh),
    )
}

impl super::ProjectileBundle for BulletProjectileConfig {
    fn projectile_bundle(&self) -> impl Bundle {
        bullet_projectile(self.clone())
    }
}

#[derive(Default)]
pub struct BulletProjectilePlugin {
    pub render: bool,
}

impl Plugin for BulletProjectilePlugin {
    fn build(&self, app: &mut App) {
        if self.render {
            app.add_observer(insert_projectile_render);
        }
    }
}

fn insert_projectile_render(
    add: On<Add, BulletProjectileMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_render_mesh: Query<&BulletProjectileRenderMesh>,
) {
    let entity = add.entity;
    debug!("Inserting BulletProjectile render for entity: {:?}", entity);
    let Ok(render_mesh) = q_render_mesh.get(entity) else {
        warn!(
            "BulletProjectile entity {:?} missing BulletProjectileRenderMesh component",
            entity
        );
        return;
    };

    match &**render_mesh {
        Some(scene_handle) => {
            commands.entity(entity).insert((children![(
                Name::new("Bullet Projectile Render"),
                SceneRoot(scene_handle.clone()),
            ),],));
        }
        None => {
            commands.entity(entity).insert((children![(
                Name::new("Bullet Projectile Render"),
                Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 1.0))),
                MeshMaterial3d(materials.add(Color::srgb(1.0, 0.9, 0.2))),
            ),],));
        }
    }
}
