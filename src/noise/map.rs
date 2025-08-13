use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use itertools::Itertools;

pub mod prelude {
    pub use super::{NoisePlugin, NoiseSet};
}

pub trait NoiseFunction<T, U, const DIM: usize> {
    fn get(&self, point: [T; DIM]) -> U;
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoiseSet;

pub struct NoisePlugin<T, U, const DIM: usize, F, Coord, Noise>
where
    F: NoiseFunction<T, U, DIM>,
    for<'a> &'a Coord: Into<[T; DIM]>,
    Noise: From<U>,
{
    func: F,
    _marker_t: std::marker::PhantomData<T>,
    _marker_u: std::marker::PhantomData<U>,
    _marker_coord: std::marker::PhantomData<Coord>,
    _marker_noise: std::marker::PhantomData<Noise>,
}

impl<T, U, const DIM: usize, F, Coord, Noise> NoisePlugin<T, U, DIM, F, Coord, Noise>
where
    F: NoiseFunction<T, U, DIM>,
    for<'a> &'a Coord: Into<[T; DIM]>,
    Noise: From<U>,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _marker_t: std::marker::PhantomData,
            _marker_u: std::marker::PhantomData,
            _marker_coord: std::marker::PhantomData,
            _marker_noise: std::marker::PhantomData,
        }
    }
}

impl<T, U, const DIM: usize, F, Coord, Noise> Plugin for NoisePlugin<T, U, DIM, F, Coord, Noise>
where
    T: Clone + Send + Sync + 'static,
    U: Clone + Send + Sync + 'static,
    F: Resource + NoiseFunction<T, U, DIM> + Clone + Send + Sync + 'static,
    for<'a> &'a Coord: Into<[T; DIM]>,
    Coord: Component + Send + Sync + 'static,
    Noise: Component + From<U> + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(self.func.clone());

        app.add_systems(
            Update,
            (
                generate_noise::<T, U, DIM, F, Coord, Noise>,
                handle_generate_noise::<U, Noise>,
            )
                .in_set(NoiseSet),
        );
    }
}

#[derive(Component)]
struct ComputeNoise<U, Noise: From<U>> {
    task: Task<CommandQueue>,
    _maker_u: std::marker::PhantomData<U>,
    _marker_noise: std::marker::PhantomData<Noise>,
}

impl<U, Noise: From<U>> ComputeNoise<U, Noise> {
    fn new(task: Task<CommandQueue>) -> Self {
        Self {
            task,
            _maker_u: std::marker::PhantomData,
            _marker_noise: std::marker::PhantomData,
        }
    }
}

#[derive(Component)]
struct ComputePoint;

fn generate_noise<T, U, const DIM: usize, F, Coord, Noise>(
    mut commands: Commands,
    func: Res<F>,
    q_point: Query<(Entity, &Coord, &ChildOf), (Without<Noise>, Without<ComputePoint>)>,
) where
    T: Clone + Send + Sync + 'static,
    U: Clone + Send + Sync + 'static,
    F: Resource + NoiseFunction<T, U, DIM> + Clone + Send + Sync + 'static,
    for<'a> &'a Coord: Into<[T; DIM]>,
    Coord: Component + Send + Sync + 'static,
    Noise: Component + From<U> + Send + Sync + 'static,
{
    let thread_pool = AsyncComputeTaskPool::get();
    for (&chunk_entity, chunk) in q_point.iter().chunk_by(|(_, _, ChildOf(e))| e).into_iter() {
        let chunk = chunk
            .map(|(child_entity, point, _)| (child_entity, point.into()))
            .collect_vec();

        for (child_entity, _) in chunk.iter() {
            commands.entity(*child_entity).insert(ComputePoint);
        }

        let func = func.clone();
        let task = thread_pool.spawn(async move {
            let mut command_queue = CommandQueue::default();
            for (child_entity, point) in chunk {
                let noise = func.get(point);
                command_queue.push(move |world: &mut World| {
                    world.entity_mut(child_entity).insert(Noise::from(noise));
                });
            }

            command_queue.push(move |world: &mut World| {
                world
                    .entity_mut(chunk_entity)
                    .remove::<ComputeNoise<U, Noise>>();
            });
            command_queue
        });

        commands
            .entity(chunk_entity)
            .insert(ComputeNoise::<U, Noise>::new(task));
    }
}

fn handle_generate_noise<U, Noise>(
    mut commands: Commands,
    mut tasks: Query<&mut ComputeNoise<U, Noise>>,
) where
    U: Send + Sync + 'static,
    Noise: From<U> + Send + Sync + 'static,
{
    for mut task in tasks.iter_mut() {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.task)) {
            commands.append(&mut commands_queue);
        }
    }
}
