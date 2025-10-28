use bevy::prelude::*;
use std::fmt::Debug;

use crate::prelude::TransformChainWorld;

pub mod prelude {
    pub use super::projectile_spawner;
    pub use super::ProjectileSpawnerConfig;
    pub use super::ProjectileSpawnerMarker;
    pub use super::ProjectileSpawnerPlugin;
    pub use super::ProjectileSpawnerSystems;
    pub use super::SpawnProjectile;
}

/// Configuration for a projectile spawner.
#[derive(Clone, Debug)]
pub struct ProjectileSpawnerConfig<T>
where
    T: Clone + Debug,
{
    /// Fire rate in shots per second.
    pub fire_rate: f32,
    /// Projectile configuration.
    pub projectile: T,
}

impl<T> Default for ProjectileSpawnerConfig<T>
where
    T: Default + Clone + Debug,
{
    fn default() -> Self {
        Self {
            fire_rate: 2.0,
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
pub struct ProjectileSpawnerFireRate(pub f32);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct ProjectileSpawnerProjectile<T>(T);

pub fn projectile_spawner<T>(config: ProjectileSpawnerConfig<T>) -> impl Bundle
where
    T: Default + Clone + Debug + Send + Sync + 'static,
{
    (
        ProjectileSpawnerMarker::<T>::default(),
        ProjectileSpawnerFireRate(config.fire_rate),
        ProjectileSpawnerProjectile(config.projectile),
    )
}

/// Event to request spawning a projectile from a spawner entity.
#[derive(Event, Debug, Clone)]
pub struct SpawnProjectile<T> {
    /// The spawner entity to spawn the projectile from.
    pub entity: Entity,
    /// Inherited Velocity to add to the spawned projectile.
    pub initial_velocity: Vec3,
    pub _marker: std::marker::PhantomData<T>,
}

impl<T> Default for SpawnProjectile<T> {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            initial_velocity: Vec3::ZERO,
            _marker: std::marker::PhantomData,
        }
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProjectileSpawnerSystems {
    Sync,
}

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
        debug!("ProjectileSpawnerPlugin: build");

        app.add_systems(
            FixedUpdate,
            update_projectile_spawners::<T>.in_set(ProjectileSpawnerSystems::Sync),
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
    trace!("on_insert_projectile_spawner: entity {:?}", entity);

    let Ok(fire_rate) = q_fire_rate.get(entity) else {
        warn!(
            "on_insert_projectile_spawner: entity {:?} not found in q_fire_rate",
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
    spawn: On<SpawnProjectile<T>>,
    mut commands: Commands,
    mut q_spawners: Query<
        (
            // NOTE: Here I would like to use GlobalTransform, but for some reason, avian3d does
            // not work properly with it...
            // TODO: Use GlobalTransform when I figure out how to fix the issue with avian3d
            &TransformChainWorld,
            &mut ProjectileSpawnerFireState,
            &ProjectileSpawnerProjectile<T>,
        ),
        With<ProjectileSpawnerMarker<T>>,
    >,
) where
    T: super::ProjectileBundle + Default + Clone + Debug + Send + Sync + 'static,
{
    let entity = spawn.entity;
    trace!("on_spawn_projectile: entity {:?}", entity);

    let Ok((spawner_transform, mut fire_state, projectile)) = q_spawners.get_mut(entity) else {
        warn!(
            "on_spawn_projectile: entity {:?} not found in q_spawners",
            entity
        );
        return;
    };

    if !fire_state.is_finished() {
        return;
    }
    debug!(
        "on_spawn_projectile: spawning projectile from entity {:?}",
        entity
    );

    let projectile_transform = Transform {
        translation: spawner_transform.translation(),
        rotation: spawner_transform.rotation(),
        ..default()
    };

    commands.spawn((
        Name::new("Projectile"),
        super::ProjectileMarker,
        super::ProjectileVelocity(spawn.initial_velocity),
        projectile_transform,
        Visibility::Visible,
        projectile.projectile_bundle(),
    ));

    fire_state.reset();
}
