use crate::{
    assets::prelude::*, controller::prelude::*, debug::prelude::*, render::prelude::*,
    states::GameStates, terrain::prelude::*,
};
use bevy::prelude::*;
use hexx::*;
use systems::hexmap::map::HexDiscoverEvent;

mod assets;
mod controller;
mod debug;
mod render;
mod states;
mod terrain;

// This is included for const, but it is unstable...
const FRAC_1_SQRT_3: f32 = 0.577350269189625764509148780501957456_f32;

const HEX_SIZE: f32 = 2.0 * FRAC_1_SQRT_3;
const CHUNK_RADIUS: u32 = 15;
const DISCOVER_RADIUS: u32 = 3;
const COLUMN_HEIGHT: f32 = 10.0;

fn main() {
    let layout = HexLayout::flat().with_hex_size(HEX_SIZE);
    let seed = 0;

    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameStates>()
        .add_plugins(AssetsPlugin)
        .add_plugins(PlanetPlugin::new(
            seed,
            layout.clone(),
            CHUNK_RADIUS,
            DISCOVER_RADIUS,
        ))
        .add_plugins(RenderPlugin::new(layout, CHUNK_RADIUS, COLUMN_HEIGHT))
        .add_plugins(WASDCameraControllerPlugin)
        .add_plugins(DebugPlugin)
        .configure_sets(
            Update,
            AssetsPluginSet.run_if(in_state(GameStates::AssetLoading)),
        )
        .configure_sets(
            Update,
            PlanetPluginSet.run_if(in_state(GameStates::Playing)),
        )
        .configure_sets(
            Update,
            RenderPluginSet.run_if(in_state(GameStates::Playing)),
        )
        .configure_sets(
            Update,
            WASDCameraControllerPluginSet.run_if(in_state(GameStates::Playing)),
        )
        .configure_sets(Update, DebugPluginSet.run_if(in_state(GameStates::Playing)))
        .add_systems(OnEnter(GameStates::Playing), setup)
        .add_systems(
            Update,
            mouse_click_discover.run_if(in_state(GameStates::Playing)),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        WASDCameraControllerBundle::default(),
        Camera3d::default(),
        Transform::from_xyz(60.0, 60.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("RTS Camera"),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(60.0, 60.0, 00.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Directional Light"),
    ));
}

// This is more for debugging purposes, to discover hexes by clicking on the map.

fn mouse_click_discover(
    windows: Query<&Window>,
    q_camera: Single<(&Camera, &GlobalTransform)>,
    mut ev_discover: EventWriter<HexDiscoverEvent<HexCoord>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let (camera, transform) = q_camera.into_inner();

    let Some(cursor_position) = windows.single().unwrap().cursor_position() else {
        return;
    };

    let Ok(ray) = camera.viewport_to_world(transform, cursor_position) else {
        return;
    };

    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else {
        return;
    };
    let point = ray.get_point(distance);

    ev_discover.write(HexDiscoverEvent::new(point.xz()));
}
