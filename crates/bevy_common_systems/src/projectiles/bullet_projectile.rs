use crate::helpers::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;

pub mod prelude {
    pub use super::bullet_projectile;
    pub use super::BulletProjectileConfig;
    pub use super::BulletProjectileMarker;
    pub use super::BulletProjectilePlugin;
    pub use super::BulletProjectilePluginSet;
    pub use super::BulletProjectileRenderMesh;
}

#[derive(Clone, Debug, Reflect)]
pub struct BulletProjectileConfig {
    pub muzzle_speed: f32,
    pub lifetime: f32,
    pub render_mesh: Option<Handle<Scene>>,
}

impl Default for BulletProjectileConfig {
    fn default() -> Self {
        Self {
            muzzle_speed: 50.0,
            lifetime: 5.0,
            render_mesh: None,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct BulletProjectileMarker;

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct BulletProjectileMuzzleSpeed(pub f32);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct BulletProjectileRenderMesh(pub Option<Handle<Scene>>);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct BulletProjectilePrev(pub Option<Vec3>);

pub fn bullet_projectile(config: BulletProjectileConfig) -> impl Bundle {
    (
        BulletProjectileMarker,
        TempEntity(config.lifetime),
        BulletProjectileMuzzleSpeed(config.muzzle_speed),
        BulletProjectileRenderMesh(config.render_mesh),
        BulletProjectilePrev(None),
    )
}

impl super::ProjectileBundle for BulletProjectileConfig {
    fn projectile_bundle(&self) -> impl Bundle {
        bullet_projectile(self.clone())
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BulletProjectilePluginSet;

#[derive(Default)]
pub struct BulletProjectilePlugin {
    pub render: bool,
}

impl Plugin for BulletProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_ray_projectiles.in_set(BulletProjectilePluginSet),
        );

        app.add_systems(
            FixedUpdate,
            update_sweep_collisions.in_set(BulletProjectilePluginSet),
        );

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
                Mesh3d(meshes.add(Cuboid::new(0.05, 0.05, 0.3))),
                MeshMaterial3d(materials.add(Color::srgb(1.0, 0.9, 0.2))),
            ),],));
        }
    }
}

fn update_ray_projectiles(
    mut q_projectiles: Query<
        (&BulletProjectileMuzzleSpeed, &mut Transform),
        With<super::ProjectileMarker>,
    >,
    time: Res<Time>,
) {
    for (speed, mut transform) in &mut q_projectiles {
        let distance = **speed * time.delta_secs();
        let forward = transform.forward();
        transform.translation += distance * *forward;
    }
}

fn update_sweep_collisions(
    mut commands: Commands,
    query: SpatialQuery,
    mut q_projectiles: Query<
        (Entity, &mut Transform, &mut BulletProjectilePrev),
        With<super::ProjectileMarker>,
    >,
) {
    let filter = SpatialQueryFilter::default();

    for (entity, transform, mut prev) in &mut q_projectiles {
        if prev.is_none() {
            **prev = Some(transform.translation);
            continue;
        }

        let origin = prev.unwrap();
        let direction = transform.translation - origin;
        **prev = Some(transform.translation);

        let Ok((direction, distance)) = Dir3::new_and_length(direction) else {
            continue;
        };

        if let Some(ray_hit_data) = query.cast_ray(origin, direction, distance, true, &filter) {
            // NOTE: For now, we just despawn the projectile
            commands.entity(entity).despawn();
        }
    }
}
