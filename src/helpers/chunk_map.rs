//! Chunk Map Plugin for Bevy
//!
//! This plugin provides a system that takes as input a query of entities, applies a function on
//! that query and inserts the result as a component on the entities. This is done using tasks
//! to allow for non-blocking computation and parallel processing.

use bevy::{
    ecs::{
        query::{QueryData, QueryItem},
        world::CommandQueue,
    },
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use itertools::Itertools;

pub mod prelude {
    pub use super::*;
}

pub trait ChunkMapInput {
    type Query: bevy::ecs::query::QueryData;
    fn from_query_item(item: QueryItem<<Self::Query as QueryData>::ReadOnly>) -> Self;
}

pub trait ChunkMapFunction<T, U> {
    fn get(&self, point: T) -> U;
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkMapPluginSet; // NOTE: Might want to parametrize this to match the plugin's type

pub struct ChunkMapPlugin<T, U, F>
where
    F: ChunkMapFunction<T, U>,
{
    func: F,
    _marker_t: std::marker::PhantomData<T>,
    _marker_u: std::marker::PhantomData<U>,
}

impl<T, U, F> ChunkMapPlugin<T, U, F>
where
    F: ChunkMapFunction<T, U>,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _marker_t: std::marker::PhantomData,
            _marker_u: std::marker::PhantomData,
        }
    }
}

impl<T, U, F> Plugin for ChunkMapPlugin<T, U, F>
where
    T: ChunkMapInput + Clone + Send + Sync + 'static,
    U: Component + Clone + Send + Sync + 'static,
    F: Resource + ChunkMapFunction<T, U> + Clone + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(self.func.clone());

        app.add_systems(
            Update,
            (create_task::<T, U, F>, handle_task::<U>).in_set(ChunkMapPluginSet),
        );
    }
}

#[derive(Component)]
struct ComputeTask<U> {
    task: Task<CommandQueue>,
    _maker_u: std::marker::PhantomData<U>,
}

impl<U> ComputeTask<U> {
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

fn create_task<T, U, F>(
    mut commands: Commands,
    func: Res<F>,
    q_point: Query<
        (Entity, <T as ChunkMapInput>::Query, &ChildOf),
        (Without<U>, Without<ComputePoint<U>>),
    >,
) where
    T: ChunkMapInput + Clone + Send + Sync + 'static,
    U: Component + Clone + Send + Sync + 'static,
    F: Resource + ChunkMapFunction<T, U> + Clone + Send + Sync + 'static,
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
            commands
                .entity(*child_entity)
                .insert(ComputePoint::<U>::new());
        }

        let func = func.clone();
        let task = thread_pool.spawn(async move {
            let mut command_queue = CommandQueue::default();
            for (child_entity, input) in chunk {
                let value = func.get(input);
                command_queue.push(move |world: &mut World| {
                    world.entity_mut(child_entity).insert(U::from(value));
                });
            }

            command_queue.push(move |world: &mut World| {
                world.entity_mut(chunk_entity).remove::<ComputeTask<U>>();
            });
            command_queue
        });

        commands
            .entity(chunk_entity)
            .insert(ComputeTask::<U>::new(task));
    }
}

fn handle_task<U>(mut commands: Commands, mut tasks: Query<&mut ComputeTask<U>>)
where
    U: Send + Sync + 'static,
{
    for mut task in tasks.iter_mut() {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.task)) {
            commands.append(&mut commands_queue);
        }
    }
}
