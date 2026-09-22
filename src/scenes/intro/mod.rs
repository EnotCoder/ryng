use bevy::prelude::*;

use crate::state::GameState;

mod systems;
mod ui;

pub struct IntroPlugin;

impl Plugin for IntroPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<systems::IntroTimer>()
            .add_systems(OnEnter(GameState::Intro), ui::spawn_intro_ui)
            .add_systems(
                Update,
                systems::intro_system.run_if(in_state(GameState::Intro)),
            );
    }
}
