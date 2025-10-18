use bevy::prelude::*;
use std::fmt::Debug;

use crate::{projectiles::prelude::*, sections::prelude::*};

pub mod prelude {
    pub use super::AttachmentPlugin;
    pub use super::AttachmentMarker;
    pub use super::WeaponAttachment;
    pub use super::weapon_attachment;
}

pub struct AttachmentPlugin;

impl Plugin for AttachmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_add_weapon_attachment::<BulletProjectileConfig>);
    }
}

#[derive(Component, Default, Clone, Debug, Reflect)]
pub struct AttachmentMarker;

#[derive(Component, Default, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct WeaponAttachment<T: Clone + Debug>(pub ProjectileSpawnerConfig<T>);

pub fn weapon_attachment<T>(config: ProjectileSpawnerConfig<T>) -> impl Bundle
where
    T: Default + Clone + Debug + Send + Sync + 'static,
{
    debug!(
        "Creating direct projectile spawner with config: {:?}",
        config
    );

    (
        Name::new("Weapon Attachment"),
        AttachmentMarker,
        WeaponAttachment::<T>(config),
    )
}

fn on_add_weapon_attachment<T>(
    add: On<Add, WeaponAttachment<T>>,
    mut commands: Commands,
    q_attachment: Query<(&WeaponAttachment<T>, &ChildOf)>,
    q_turret: Query<&TurretSectionBarrelMuzzleEntity, With<TurretSectionMarker>>,
) where
    T: ProjectileBundle + Default + Clone + Debug + Send + Sync + 'static,
{
    let entity = add.entity;
    debug!("Inserting WeaponAttachment: {:?}", entity);
    let Ok((attachment, ChildOf(entity))) = q_attachment.get(entity) else {
        warn!(
            "WeaponAttachment entity {:?} missing WeaponAttachment component",
            entity
        );
        return;
    };

    let Ok(muzzle) = q_turret.get(*entity) else {
        warn!(
            "WeaponAttachment entity {:?} is not a child of a TurretSectionMarker",
            entity
        );
        return;
    };

    commands.entity(**muzzle).with_children(|parent| {
        parent.spawn(projectile_spawner((**attachment).clone()));
    });
}
