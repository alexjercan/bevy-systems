use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use bevy_rand::prelude::*;
use rand::RngCore;

use crate::components::EntityTypeName;

pub mod prelude {
    pub use super::{
        asteroid_scenario_object, AsteroidConfig, AsteroidMarker, AsteroidPlugin, AsteroidRadius,
        AsteroidRenderMesh, AsteroidTexture, ASTEROID_TYPE_NAME,
    };
}

pub const ASTEROID_TYPE_NAME: &str = "asteroid";

#[derive(Clone, Debug)]
pub struct AsteroidConfig {
    pub radius: f32,
    pub texture: Handle<Image>,
}

pub fn asteroid_scenario_object(config: AsteroidConfig) -> impl Bundle {
    debug!("asteroid_scenario_object: config {:?}", config);

    (
        AsteroidMarker,
        EntityTypeName::new(ASTEROID_TYPE_NAME),
        AsteroidTexture(config.texture),
        AsteroidRadius(config.radius),
    )
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct AsteroidMarker;

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct AsteroidTexture(pub Handle<Image>);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct AsteroidRenderMesh(pub Mesh);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct AsteroidRadius(pub f32);

pub struct AsteroidPlugin {
    pub render: bool,
}

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        debug!("AsteroidPlugin: build");

        app.add_observer(insert_asteroid_collider);
        if self.render {
            app.add_observer(insert_asteroid_render);
        }
    }
}

fn insert_asteroid_collider(
    add: On<Add, AsteroidMarker>,
    mut commands: Commands,
    q_asteroid: Query<&AsteroidRadius, With<AsteroidMarker>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    let entity = add.entity;
    trace!("insert_asteroid_render: entity {:?}", entity);

    let Ok(radius) = q_asteroid.get(entity) else {
        warn!(
            "insert_asteroid_render: entity {:?} not found in q_asteroid",
            entity
        );
        return;
    };

    let mesh = apply_noise_to_mesh(&octahedron_sphere(3), rng.next_u32());
    let collider = Collider::trimesh_from_mesh(&mesh).unwrap_or(Collider::sphere(1.0));

    commands.entity(entity).insert((children![(
        Transform::from_scale(Vec3::splat(**radius)),
        AsteroidRenderMesh(mesh.clone()),
        collider,
        ColliderDensity(1.0),
        Visibility::Inherited,
    )],));
}

fn insert_asteroid_render(
    add: On<Add, AsteroidRenderMesh>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_render: Query<(&AsteroidRenderMesh, &ChildOf)>,
    q_asteroid: Query<&AsteroidTexture, With<AsteroidMarker>>,
) {
    let entity = add.entity;
    trace!("insert_asteroid_render: entity {:?}", entity);

    let Ok((render_mesh, ChildOf(asteroid))) = q_render.get(entity) else {
        warn!(
            "insert_asteroid_render: entity {:?} not found in q_render",
            entity
        );
        return;
    };

    let Ok(texture) = q_asteroid.get(*asteroid) else {
        warn!(
            "insert_asteroid_render: entity {:?} not found in q_asteroid",
            entity
        );
        return;
    };

    let mesh = (**render_mesh).clone();
    let material = StandardMaterial {
        base_color_texture: Some((**texture).clone()),
        ..default()
    };

    commands.entity(entity).insert((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
    ));
}
