use bevy::prelude::*;
use std::fmt::Debug;

pub mod prelude {
    pub use super::projectile_spawner;
    pub use super::ProjectileSpawnerConfig;
    pub use super::ProjectileSpawnerMarker;
    pub use super::ProjectileSpawnerPlugin;
    pub use super::SpawnProjectile;
}

#[derive(Clone, Debug)]
pub struct ProjectileSpawnerConfig<T>
where
    T: Clone + Debug,
{
    pub muzzle_offset: Vec3,
    pub fire_rate: f32,
    pub transform: Transform,
    pub projectile: T,
}

impl<T> Default for ProjectileSpawnerConfig<T>
where
    T: Default + Clone + Debug,
{
    fn default() -> Self {
        Self {
            muzzle_offset: Vec3::ZERO,
            fire_rate: 2.0,
            transform: Transform::default(),
            projectile: T::default(),
        }
    }
}

#[derive(Component, Default, Clone, Debug, Reflect)]
pub struct ProjectileSpawnerMarker<T>
where
    T: Default + Clone + Debug + 'static,
{
    _marker: std::marker::PhantomData<T>,
}

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct ProjectileSpawnerMuzzleOffset(pub Vec3);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct ProjectileSpawnerFireRate(pub f32);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct ProjectileSpawnerProjectile<T>(T);

pub fn projectile_spawner<T>(config: ProjectileSpawnerConfig<T>) -> impl Bundle
where
    T: Default + Clone + Debug + Send + Sync + 'static,
{
    debug!(
        "Creating direct projectile spawner with config: {:?}",
        config
    );

    (
        Name::new("Projectile Spawner"),
        ProjectileSpawnerMarker::<T>::default(),
        ProjectileSpawnerMuzzleOffset(config.muzzle_offset),
        ProjectileSpawnerFireRate(config.fire_rate),
        ProjectileSpawnerProjectile(config.projectile),
        config.transform,
        Visibility::Visible,
    )
}

#[derive(Event)]
pub struct SpawnProjectile {
    pub entity: Entity,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectileSpawnerPluginSet;

#[derive(Default)]
pub struct ProjectileSpawnerPlugin<T>
where
    T: Default,
{
    _marker: std::marker::PhantomData<T>,
}

impl<T> Plugin for ProjectileSpawnerPlugin<T>
where
    T: super::ProjectileBundle + Default + Clone + Debug + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            update_projectile_spawners::<T>.in_set(ProjectileSpawnerPluginSet),
        );

        app.add_observer(on_insert_projectile_spawner::<T>);
        app.add_observer(on_spawn_projectile::<T>);
    }
}

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct ProjectileSpawnerFireState(Timer);

fn update_projectile_spawners<T>(
    mut q_spawners: Query<&mut ProjectileSpawnerFireState, With<ProjectileSpawnerMarker<T>>>,
    time: Res<Time>,
) where
    T: Default + Clone + Debug + Send + Sync + 'static,
{
    for mut fire_state in &mut q_spawners {
        fire_state.tick(time.delta());
    }
}

fn on_insert_projectile_spawner<T>(
    insert: On<Insert, ProjectileSpawnerMarker<T>>,
    mut commands: Commands,
    q_fire_rate: Query<&ProjectileSpawnerFireRate>,
) where
    T: Default + Clone + Debug + Send + Sync + 'static,
{
    let entity = insert.entity;
    debug!("Inserting ProjectileSpawner: {:?}", entity);
    let Ok(fire_rate) = q_fire_rate.get(entity) else {
        warn!(
            "ProjectileSpawner entity {:?} missing ProjectileSpawnerFireRate component",
            entity
        );
        return;
    };

    let interval = 1.0 / **fire_rate;
    let mut timer = Timer::from_seconds(interval, TimerMode::Once);
    timer.finish(); // Ready to fire immediately

    commands
        .entity(entity)
        .insert(ProjectileSpawnerFireState(timer));
}

fn on_spawn_projectile<T>(
    spawn: On<SpawnProjectile>,
    mut commands: Commands,
    mut q_spawners: Query<
        (
            &GlobalTransform,
            &ProjectileSpawnerMuzzleOffset,
            &mut ProjectileSpawnerFireState,
            &ProjectileSpawnerProjectile<T>,
        ),
        With<ProjectileSpawnerMarker<T>>,
    >,
) where
    T: super::ProjectileBundle + Default + Clone + Debug + Send + Sync + 'static,
{
    let entity = spawn.entity;
    let Ok((spawner_transform, muzzle_offset, mut fire_state, projectile)) =
        q_spawners.get_mut(entity)
    else {
        warn!(
            "ProjectileSpawner entity {:?} missing required components",
            entity
        );
        return;
    };

    if !fire_state.is_finished() {
        trace!(
            "ProjectileSpawner entity {:?} fire rate limit, cannot fire yet",
            entity
        );
        return;
    }

    debug!(
        "Spawning direct projectile from spawner entity: {:?}",
        entity
    );

    let matrix = spawner_transform.to_matrix();
    let offset_world = matrix.transform_point3(**muzzle_offset);
    let rotation = spawner_transform.rotation();

    let projectile_transform = Transform {
        translation: offset_world,
        rotation,
        ..default()
    };

    commands.spawn((
        Name::new("Projectile"),
        super::ProjectileMarker,
        projectile_transform,
        Visibility::Visible,
        projectile.projectile_bundle(),
    ));

    fire_state.reset();
}
