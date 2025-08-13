// TODO: Documentation for this module; rethink the name, it is more of a Chunked Multithreaded Map
// system rather than a Noise system; maybe it can go into a `util` module and then this would be
// called `util::chunked_map` or something similar.

use bevy::{
    ecs::{query::{QueryData, QueryItem}, world::CommandQueue},
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use itertools::Itertools;

pub mod prelude {
    pub use super::{NoisePlugin, NoiseSet};
}

pub trait NoiseInput {
    type Query: bevy::ecs::query::QueryData;
    fn from_query_item(item: QueryItem<<Self::Query as QueryData>::ReadOnly>) -> Self;
}

pub trait NoiseFunction<T, U> {
    fn get(&self, point: T) -> U;
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoiseSet;

pub struct NoisePlugin<T, U, F>
where
    F: NoiseFunction<T, U>,
{
    func: F,
    _marker_t: std::marker::PhantomData<T>,
    _marker_u: std::marker::PhantomData<U>,
}

impl<T, U, F> NoisePlugin<T, U, F>
where
    F: NoiseFunction<T, U>,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _marker_t: std::marker::PhantomData,
            _marker_u: std::marker::PhantomData,
        }
    }
}

impl<T, U, F> Plugin for NoisePlugin<T, U, F>
where
    T: NoiseInput + Clone + Send + Sync + 'static,
    U: Component + Clone + Send + Sync + 'static,
    F: Resource + NoiseFunction<T, U> + Clone + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(self.func.clone());

        app.add_systems(
            Update,
            (
                generate_noise::<T, U, F>,
                handle_generate_noise::<U>,
            )
                .in_set(NoiseSet),
        );
    }
}

#[derive(Component)]
struct ComputeNoise<U> {
    task: Task<CommandQueue>,
    _maker_u: std::marker::PhantomData<U>,
}

impl<U> ComputeNoise<U> {
    fn new(task: Task<CommandQueue>) -> Self {
        Self {
            task,
            _maker_u: std::marker::PhantomData,
        }
    }
}

#[derive(Component)]
struct ComputePoint<U> {
    _marker_u: std::marker::PhantomData<U>,
}

impl<U> ComputePoint<U> {
    fn new() -> Self {
        Self {
            _marker_u: std::marker::PhantomData,
        }
    }
}

fn generate_noise<T, U, F>(
    mut commands: Commands,
    func: Res<F>,
    q_point: Query<(Entity, <T as NoiseInput>::Query, &ChildOf), (Without<U>, Without<ComputePoint<U>>)>,
) where
    T: NoiseInput + Clone + Send + Sync + 'static,
    U: Component + Clone + Send + Sync + 'static,
    F: Resource + NoiseFunction<T, U> + Clone + Send + Sync + 'static,
{
    let thread_pool = AsyncComputeTaskPool::get();
    for (&chunk_entity, chunk) in q_point.iter().chunk_by(|(_, _, ChildOf(e))| e).into_iter() {
        let chunk = chunk
            .map(|(child_entity, query_data, _)| {
                let input = T::from_query_item(query_data);
                (child_entity, input)
            })
            .collect_vec();

        for (child_entity, _) in chunk.iter() {
            commands.entity(*child_entity).insert(ComputePoint::<U>::new());
        }

        let func = func.clone();
        let task = thread_pool.spawn(async move {
            let mut command_queue = CommandQueue::default();
            for (child_entity, input) in chunk {
                let noise = func.get(input);
                command_queue.push(move |world: &mut World| {
                    world.entity_mut(child_entity).insert(U::from(noise));
                });
            }

            command_queue.push(move |world: &mut World| {
                world
                    .entity_mut(chunk_entity)
                    .remove::<ComputeNoise<U>>();
            });
            command_queue
        });

        commands
            .entity(chunk_entity)
            .insert(ComputeNoise::<U>::new(task));
    }
}

fn handle_generate_noise<U>(
    mut commands: Commands,
    mut tasks: Query<&mut ComputeNoise<U>>,
) where
    U: Send + Sync + 'static,
{
    for mut task in tasks.iter_mut() {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.task)) {
            commands.append(&mut commands_queue);
        }
    }
}
