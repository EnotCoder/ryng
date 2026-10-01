use bevy::prelude::*;

use crate::acts::Item;
use crate::state::GameState;

mod ui;

#[cfg(test)]
mod tests;

pub use ui::ActiveInvSlot;

pub struct InventoryUiPlugin;

impl Plugin for InventoryUiPlugin {
    fn build(&self, app: &mut App) {
        let textures = {
            let world = app.world_mut();
            let asset_server = world.resource::<AssetServer>();
            ui::InventoryTextures {
                active_slot: asset_server.load("tex/ui/inv_active_slot.png"),
                disabled_slot: asset_server.load("tex/ui/inv_disable_slot.png"),
                // Every item is listed here, so an `Item` without an icon is a
                // compile error rather than an empty slot in game.
                icons: [
                    Item::Pass,
                    Item::MainKey,
                    Item::Teddy,
                    Item::Crowbar,
                    Item::MetalCutters,
                    Item::KeyDoor2,
                ]
                .into_iter()
                .map(|item| (item, asset_server.load(item.icon_path())))
                .collect(),
            }
        };
        app.insert_resource(textures)
            .init_resource::<ui::ActiveInvSlot>()
            .add_systems(OnEnter(GameState::Game), ui::spawn_inventory_ui)
            .add_systems(
                Update,
                // The editor takes the pointer, so slot clicks stand down with
                // the rest of the gameplay handling.
                (
                    ui::slot_click_system.run_if(super::gameplay_active),
                    ui::update_inventory_ui,
                )
                    .run_if(in_state(GameState::Game)),
            );
    }
}
