use bevy::prelude::*;

use crate::state::GameState;

mod systems;
mod ui;

pub use ui::MenuAction;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), ui::spawn_menu_ui)
            .add_systems(Update, systems::menu_button_system);
    }
}
