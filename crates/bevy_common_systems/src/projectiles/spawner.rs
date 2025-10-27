use bevy::prelude::*;
use std::fmt::Debug;

pub mod prelude {
    pub use super::projectile_spawner;
    pub use super::ProjectileSpawnerConfig;
    pub use super::ProjectileSpawnerMarker;
    pub use super::ProjectileSpawnerPlugin;
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
    debug!("Creating projectile spawner with config: {:?}", config);

    (
        ProjectileSpawnerMarker::<T>::default(),
        ProjectileSpawnerFireRate(config.fire_rate),
        ProjectileSpawnerProjectile(config.projectile),
    )
}

/// Event to request spawning a projectile from a spawner entity.
#[derive(Event, Debug, Clone)]
pub struct SpawnProjectile {
    /// The spawner entity to spawn the projectile from.
    pub entity: Entity,
    /// Inherited Velocity to add to the spawned projectile.
    pub initial_velocity: Vec3,
    /// Spawn position.
    pub spawn_position: Vec3,
    /// Spawn rotation.
    pub spawn_rotation: Quat,
}

impl Default for SpawnProjectile {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            initial_velocity: Vec3::ZERO,
            spawn_position: Vec3::ZERO,
            spawn_rotation: Quat::IDENTITY,
        }
    }
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
            &mut ProjectileSpawnerFireState,
            &ProjectileSpawnerProjectile<T>,
        ),
        With<ProjectileSpawnerMarker<T>>,
    >,
) where
    T: super::ProjectileBundle + Default + Clone + Debug + Send + Sync + 'static,
{
    let entity = spawn.entity;
    let Ok((mut fire_state, projectile)) = q_spawners.get_mut(entity) else {
        warn!(
            "ProjectileSpawner entity {:?} missing required components",
            entity
        );
        return;
    };

    if !fire_state.is_finished() {
        return;
    }
    debug!("Spawning projectile from spawner entity: {:?}", entity);

    let projectile_transform = Transform {
        translation: spawn.spawn_position,
        rotation: spawn.spawn_rotation,
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
