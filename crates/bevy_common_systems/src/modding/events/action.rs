use bevy::prelude::*;

pub trait EventAction: std::fmt::Debug + Send + Sync {
    fn action(&self, commands: &mut Commands);

    fn name(&self) -> &'static str { std::any::type_name::<Self>() }
}
