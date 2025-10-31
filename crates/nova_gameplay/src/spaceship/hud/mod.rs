use bevy::prelude::*;

pub mod health;
pub mod velocity;

pub mod prelude {
    pub use super::{health::prelude::*, velocity::prelude::*, SpacehipHudPlugin};
}

#[derive(Default)]
pub struct SpacehipHudPlugin;

impl Plugin for SpacehipHudPlugin {
    fn build(&self, app: &mut App) {
        debug!("HudPlugin: build");

        app.add_plugins(velocity::VelocityHudPlugin);
        app.add_plugins(health::HealthHudPlugin);
    }
}
