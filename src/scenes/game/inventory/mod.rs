use bevy::prelude::*;

use crate::state::GameState;

mod ui;

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
                icon_pass: asset_server.load("tex/ui/icons_inv/kon_card.png"),
                icon_main_key: asset_server.load("tex/ui/icons_inv/main_key.png"),
            }
        };
        app.insert_resource(textures)
            .init_resource::<ui::ActiveInvSlot>()
            .add_systems(OnEnter(GameState::Game), ui::spawn_inventory_ui)
            .add_systems(
                Update,
                (ui::slot_click_system, ui::update_inventory_ui).run_if(in_state(GameState::Game)),
            );
    }
}
