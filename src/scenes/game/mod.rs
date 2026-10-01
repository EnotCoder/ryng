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

// Compiled out entirely without the feature, so none of this can reach a
// release build.
#[cfg(feature = "hotspot-editor")]
mod hotspot_edit;

/// False while the hotspot editor is holding the pointer, so the game's own
/// click handling stands down.
///
/// Without the feature this is always true and nothing else in the crate has to
/// know the editor exists. With it, the editor always wins: a click selects a
/// hotspot rather than walking through it, which is the whole point.
pub fn gameplay_active() -> bool {
    !cfg!(feature = "hotspot-editor")
}

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
            .add_systems(OnEnter(GameState::Game), ui::spawn_game_ui);

        #[cfg(feature = "hotspot-editor")]
        app.init_resource::<hotspot_edit::Edit>()
            .insert_resource(hotspot_edit::init_clipboard())
            .add_systems(OnEnter(GameState::Game), hotspot_edit::spawn_overlay)
            .add_systems(
                Update,
                (
                    hotspot_edit::keyboard,
                    hotspot_edit::navigate,
                    hotspot_edit::clipboard_system,
                    hotspot_edit::markers,
                    hotspot_edit::readout_text,
                )
                    .chain()
                    .run_if(in_state(GameState::Game)),
            );

        app.add_systems(
            Update,
            (
                // One system per fade phase; only the one matching the
                // current phase does anything, and the order is fixed.
                auto_next_system,
                fade_out_system,
                fade_in_system,
                systems::game_button_system.run_if(gameplay_active),
                systems::carousel_system.run_if(gameplay_active),
                // While the editor is up, a click means "select this hotspot",
                // not "go through it". Both systems read the same
                // Pointer<Click> and a MessageReader does not consume it, so the
                // game's handling has to be switched off rather than out-raced.
                systems::game_hotspot_system.run_if(gameplay_active),
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
