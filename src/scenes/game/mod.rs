use bevy::prelude::*;

use crate::acts::{CurrentAct, Inventory, Item};
use crate::scenes::fade::{RoomFade, auto_next_system, fade_in_system, fade_out_system};
use crate::state::GameState;

mod inventory;
mod items;
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
            .insert_resource(items::WorldItems::with_resting([
                // The teddy starts on the basement floor; the player walks past
                // it on the way down and has to double back to notice it. The
                // three tools are in the floor 2 apartments.
                (
                    Item::Teddy,
                    "tex/rooms/basement/basement_stairs_center_room.png",
                ),
                (Item::Crowbar, "tex/rooms/floor_2/ap_1.png"),
                (Item::MetalCutters, "tex/rooms/floor_2/ap_2.png"),
                (Item::KeyDoor2, "tex/rooms/floor_2/ap_3.png"),
            ]))
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
                    // After the hotspot systems, so a pickup despawns the sprite
                    // and clears the spot in the same frame it is taken.
                    systems::item_hotspot_visibility_system,
                    systems::item_sprites_system,
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
