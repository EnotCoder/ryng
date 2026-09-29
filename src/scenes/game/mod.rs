use bevy::prelude::*;

use crate::acts::{CurrentAct, Inventory};
use crate::scenes::fade::{RoomFade, auto_next_system, fade_in_system, fade_out_system};
use crate::state::GameState;

mod inventory;
pub mod rooms;
mod systems;
#[cfg(test)]
mod tests;
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
                    // One system per fade phase; only the one matching the
                    // current phase does anything, and the order is fixed.
                    auto_next_system,
                    fade_out_system,
                    fade_in_system,
                    systems::game_button_system,
                    systems::carousel_system,
                    systems::game_hotspot_system,
                    ui::update_room_label,
                    systems::idle_breathe_system,
                    systems::blink_hotspot_icons,
                    crate::scenes::sound::background_music_system,
                )
                    .chain()
                    .run_if(in_state(GameState::Game)),
            );
    }
}
