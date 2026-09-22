use bevy::prelude::*;

use crate::acts::{CurrentAct, Inventory};
use crate::scenes::fade::{RoomFade, room_fade_system};
use crate::state::GameState;

mod inventory;
pub mod rooms;
mod systems;
mod ui;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoomFade>()
            .init_resource::<Inventory>()
            .init_resource::<CurrentAct>()
            .add_plugins(inventory::InventoryUiPlugin)
            .add_systems(OnEnter(GameState::Game), ui::spawn_game_ui)
            .add_systems(
                Update,
                (
                    systems::game_button_system,
                    systems::carousel_system,
                    systems::game_hotspot_system,
                    room_fade_system,
                    ui::update_room_label,
                )
                    .run_if(in_state(GameState::Game)),
            );
    }
}
