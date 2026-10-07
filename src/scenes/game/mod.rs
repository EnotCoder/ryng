use bevy::prelude::*;

use crate::acts::{CurrentAct, Inventory, Item};
use crate::cli;
use crate::scenes::fade::{RoomFade, auto_next_system, fade_in_system, fade_out_system};
use crate::state::GameState;

mod inventory;
mod items;
mod npc;
pub mod rooms;
mod systems;
#[cfg(test)]
mod tests;
mod ui;

/// The room the game should open in, when `--rooms` asked for one.
///
/// `None` for an ordinary run, which is what makes the flag inert by default: the
/// start room then comes from the current act as it always did.
#[derive(Resource, Default, Clone, Copy)]
pub struct StartRoom(pub Option<&'static str>);

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Read once, here, rather than in `main`: the plugin already owns the
        // starting `CurrentAct` and the room it goes with, and `init_resource`
        // would run *after* an `insert_resource` that came before it, so an
        // override set outside this chain would be quietly overwritten. Deciding
        // it in the same place as the defaults it replaces is what keeps the two
        // from disagreeing.
        let start = cli::from_args();

        app.init_resource::<RoomFade>()
            .init_resource::<Inventory>()
            .init_resource::<CurrentAct>()
            // The hover outline fades between states rather than snapping, so it
            // needs somewhere to keep how far along it is between frames.
            .init_resource::<systems::OutlineFades>()
            .insert_resource(StartRoom(start.map(|here| here.room)))
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
            .add_plugins((inventory::InventoryUiPlugin, npc::NpcPlugin))
            .add_systems(OnEnter(GameState::Game), ui::spawn_game_ui);

        // The act the flag's room belongs to, after the `init_resource` above, so
        // it wins. Without this a room in act 2 would open with `CurrentAct` still
        // on act 1, and the concierge would be lit when the route would have found
        // it dark.
        if let Some(here) = start {
            app.insert_resource(CurrentAct(here.act));
        }

        app.add_systems(
            Update,
            (
                // One system per fade phase; only the one matching the
                // current phase does anything, and the order is fixed.
                auto_next_system,
                fade_out_system,
                fade_in_system,
                systems::game_button_system,
                systems::carousel_system,
                // After the carousel system, because that is what starts a flip: the
                // room carries `RoomFlip` from the moment the arrow is pressed, and
                // this only has something to advance once it does.
                systems::room_flip_system,
                // After both, so the picture on the control swaps when the player
                // arrives at a shot rather than when they set off towards it: a press
                // starts the pan and leaves the shot alone.
                systems::carousel_control_system,
                systems::game_hotspot_system,
                // After the hotspot systems, so a pickup despawns the sprite
                // and clears the spot in the same frame it is taken.
                systems::item_hotspot_visibility_system,
                systems::item_sprites_system,
                ui::update_room_label,
                systems::idle_breathe_system,
                systems::blink_hotspot_icons,
                // After the hotspot systems, so an outline that is already lit when
                // the player arrives at a door does not get its first frame stolen
                // by the click that got them there.
                systems::hover_outline_system,
                crate::scenes::sound::background_music_system,
            )
                .chain()
                .run_if(in_state(GameState::Game)),
        );
    }
}
