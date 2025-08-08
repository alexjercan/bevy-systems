use bevy::prelude::*;
use noise::NoiseFn;

pub mod prelude {
    pub use super::{FromNoise, NoisePlugin, NoiseSet, ToNoisePoint};
}

pub trait FromNoise {
    fn from_noise(value: f64) -> Self;
}

pub trait ToNoisePoint<const DIM: usize> {
    fn to_point(&self) -> [f64; DIM];
}

#[derive(Resource, Debug, Clone, Deref, DerefMut)]
struct NoiseGenerator<const DIM: usize, F: NoiseFn<f64, DIM>>(F);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoiseSet;

pub struct NoisePlugin<const DIM: usize, T: ToNoisePoint<DIM>, F: NoiseFn<f64, DIM>, C: FromNoise> {
    func: F,
    _marker_in: std::marker::PhantomData<T>,
    _marker_out: std::marker::PhantomData<C>,
}

impl<const DIM: usize, T: ToNoisePoint<DIM>, F: NoiseFn<f64, DIM>, C: FromNoise>
    NoisePlugin<DIM, T, F, C>
{
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
        T: Component + ToNoisePoint<DIM> + Send + Sync + 'static,
        F: NoiseFn<f64, DIM> + Copy + Send + Sync + 'static,
        C: Component + FromNoise + Send + Sync + 'static,
    > Plugin for NoisePlugin<DIM, T, F, C>
{
    fn build(&self, app: &mut App) {
        app.insert_resource(NoiseGenerator(self.func));

        app.add_systems(
            Update,
            (generate_noise::<DIM, T, F, C>).in_set(NoiseSet).chain(),
        );
    }
}

fn generate_noise<
    const DIM: usize,
    T: Component + ToNoisePoint<DIM> + Send + Sync + 'static,
    F: NoiseFn<f64, DIM> + Send + Sync + 'static,
    C: Component + FromNoise + Send + Sync + 'static,
>(
    mut commands: Commands,
    generator: Res<NoiseGenerator<DIM, F>>,
    q_hex: Query<(Entity, &T), Without<C>>,
) {
    for (entity, coord) in q_hex.iter() {
        let noise = generator.get(coord.to_point());
        commands.entity(entity).insert(C::from_noise(noise));
    }
}
