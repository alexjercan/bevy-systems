//! The simulation plugin. This plugin should contain all the gameplay related logic.

use avian3d::prelude::*;
use bevy::prelude::*;
use nova_assets::prelude::*;
use nova_gameplay::prelude::*;
use rand::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationSystems;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Might want to use observers more for spawning things to avoid ordering issues

        app.add_systems(
            OnEnter(super::GameStates::Simulation),
            (
                switch_scene_on_no_player.run_if(run_once),
            ),
        );

        // On F1 we switch to editor
        // TODO: Use the input system for this
        app.add_systems(
            Update,
            switch_scene_editor.run_if(in_state(super::GameStates::Simulation)),
        );

        app.add_systems(
            OnExit(super::GameStates::Simulation),
            |mut q_thruster: Query<&mut ThrusterSectionInput, With<SpaceshipThrusterInputKey>>| {
                for mut input in &mut q_thruster {
                    **input = 0.0;
                }
            },
        );
        app.add_systems(
            OnExit(super::GameStates::Simulation),
            |mut commands: Commands, q_fragment: Query<Entity, With<FragmentMeshMarker>>| {
                for fragment in &q_fragment {
                    commands.entity(fragment).despawn();
                }
            },
        );
    }
}

fn switch_scene_editor(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<super::GameStates>>,
) {
    if keys.just_pressed(KeyCode::F1) {
        debug!("switch_scene_editor: F1 pressed, switching to Editor state.");
        state.set(super::GameStates::Editor);
    }
}

fn switch_scene_on_no_player(
    mut state: ResMut<NextState<super::GameStates>>,
    q_spaceship: Query<&Health, (With<SpaceshipRootMarker>, With<PlayerSpaceshipMarker>)>,
) {
    if q_spaceship.is_empty() {
        debug!("switch_scene_on_no_player: No player spaceship found, switching to Editor state.");
        state.set(super::GameStates::Editor);
    }
}
