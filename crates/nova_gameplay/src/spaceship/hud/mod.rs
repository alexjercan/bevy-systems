use bevy::prelude::*;

pub mod velocity;
pub mod health;

pub mod prelude {
    pub use super::velocity::prelude::*;
    pub use super::health::prelude::*;

    pub use super::HudPlugin;
    pub use super::HudPluginSet;
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HudPluginSet;

#[derive(Default)]
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(velocity::VelocityHudPlugin);
        app.add_plugins(health::HealthHudPlugin);

        app.configure_sets(Update, velocity::VelocityHudPluginSet.in_set(HudPluginSet));
        app.configure_sets(Update, health::HealthHudPluginSet.in_set(HudPluginSet));
    }
}
