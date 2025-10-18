//! Gameplay related functionality for Nova Protocol.

use avian3d::prelude::*;
use bevy::prelude::*;

pub mod spaceship;
pub mod projectile_damage;
pub mod sections;
pub mod destruction;

pub mod prelude {
    pub use super::sections::prelude::*;
    pub use super::projectile_damage::prelude::*;
    pub use super::spaceship::prelude::*;
    pub use super::destruction::prelude::*;

    pub use super::GameplayPlugin;
}

/// A system set that will contain all the systems related to the gameplay plugin.
#[derive(Default, Clone, Debug)]
pub struct GameplayPlugin {
    pub render: bool,
}

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        // We need to enable the physics plugins to have access to RigidBody and other components.
        // We will also disable gravity for this example, since we are in space, duh.
        app.add_plugins(PhysicsPlugins::default().set(PhysicsInterpolationPlugin::interpolate_all()));
        app.add_plugins(PhysicsPickingPlugin);
        app.insert_resource(Gravity::ZERO);

        // Bevy Common Systems - WASD Camera
        app.add_plugins(bevy_common_systems::prelude::WASDCameraPlugin);
        app.add_plugins(bevy_common_systems::prelude::WASDCameraControllerPlugin);
        // Chase Camera Plugin to have a 3rd person camera following the spaceship
        app.add_plugins(bevy_common_systems::prelude::ChaseCameraPlugin);
        // Bevy Common Systems - Rendering
        app.add_plugins(bevy_common_systems::prelude::SkyboxPlugin);
        app.add_plugins(bevy_common_systems::prelude::PostProcessingDefaultPlugin);
        // Point Rotation Plugin to convert linear movement to a target rotation
        app.add_plugins(bevy_common_systems::prelude::PointRotationPlugin);
        // for debug to have a random orbiting object
        app.add_plugins(bevy_common_systems::prelude::SphereRandomOrbitPlugin);
        // Rotation Plugin for the turret facing direction
        app.add_plugins(bevy_common_systems::prelude::SmoothLookRotationPlugin);
        // Other helper plugins
        app.add_plugins(bevy_common_systems::prelude::TempEntityPlugin);
        // Core Mechanics
        app.add_plugins(bevy_common_systems::prelude::ProjectilePlugin { render: true });
        app.add_plugins(bevy_common_systems::prelude::HealthPlugin);

        // Glue Plugins
        app.add_plugins(spaceship::SpaceshipPlugin { render: self.render });
        app.add_plugins(projectile_damage::ProjectileDamageGluePlugin);
        app.add_plugins(destruction::DestructionHealthPlugin);
    }
}
