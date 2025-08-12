use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use itertools::Itertools;
use noise::NoiseFn;

pub mod prelude {
    pub use super::{NoisePlugin, NoiseSet};
}

#[derive(Resource, Debug, Clone, Deref, DerefMut)]
struct NoiseGenerator<const DIM: usize, F: NoiseFn<f64, DIM> + Clone>(F);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoiseSet;

pub struct NoisePlugin<const DIM: usize, T, F, C> {
    func: F,
    _marker_in: std::marker::PhantomData<T>,
    _marker_out: std::marker::PhantomData<C>,
}

impl<const DIM: usize, T, F, C> NoisePlugin<DIM, T, F, C> {
    pub fn new(func: F) -> Self {
        Self {
            func,
            _marker_in: std::marker::PhantomData,
            _marker_out: std::marker::PhantomData,
        }
    }
}

impl<
        const DIM: usize,
        T: Component + Send + Sync + 'static,
        F: NoiseFn<f64, DIM> + Copy + Send + Sync + 'static,
        C: Component + From<f64> + Send + Sync + 'static,
    > Plugin for NoisePlugin<DIM, T, F, C>
where
    for<'a> &'a T: Into<[f64; DIM]>,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(NoiseGenerator(self.func));

        app.add_systems(
            Update,
            (generate_noise::<DIM, T, F, C>, handle_generate_noise::<C>).in_set(NoiseSet),
        );
    }
}

#[derive(Component)]
struct ComputeNoise<C: From<f64>> {
    task: Task<CommandQueue>,
    _marker: std::marker::PhantomData<C>,
}

impl<C: From<f64>> ComputeNoise<C> {
    fn new(task: Task<CommandQueue>) -> Self {
        Self {
            task,
            _marker: std::marker::PhantomData,
        }
    }
}

#[derive(Component)]
struct ComputePoint;

fn generate_noise<
    const DIM: usize,
    T: Component + Send + Sync + 'static,
    F: NoiseFn<f64, DIM> + Clone + Send + Sync + 'static,
    C: Component + From<f64> + Send + Sync + 'static,
>(
    mut commands: Commands,
    generator: Res<NoiseGenerator<DIM, F>>,
    q_point: Query<(Entity, &T, &ChildOf), (Without<C>, Without<ComputePoint>)>,
) where
    for<'a> &'a T: Into<[f64; DIM]>,
{
    let thread_pool = AsyncComputeTaskPool::get();
    for (&chunk_entity, chunk) in q_point.iter().chunk_by(|(_, _, ChildOf(e))| e).into_iter() {
        let chunk = chunk
            .map(|(child_entity, point, _)| (child_entity, point.into()))
            .collect_vec();

        for (child_entity, _) in chunk.iter() {
            commands.entity(*child_entity).insert(ComputePoint);
        }

        let generator = generator.clone();
        let task = thread_pool.spawn(async move {
            let mut command_queue = CommandQueue::default();
            for (child_entity, point) in chunk {
                let noise = generator.get(point);
                command_queue.push(move |world: &mut World| {
                    world.entity_mut(child_entity).insert(C::from(noise));
                });
            }

            command_queue.push(move |world: &mut World| {
                world.entity_mut(chunk_entity).remove::<ComputeNoise<C>>();
            });
            command_queue
        });

        commands
            .entity(chunk_entity)
            .insert(ComputeNoise::<C>::new(task));
    }
}

fn handle_generate_noise<C: Component + From<f64> + Send + Sync + 'static>(
    mut commands: Commands,
    mut tasks: Query<&mut ComputeNoise<C>>,
) {
    for mut task in tasks.iter_mut() {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.task)) {
            commands.append(&mut commands_queue);
        }
    }
}
