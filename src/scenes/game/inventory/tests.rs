//! Tests for the inventory layout.

use crate::acts::Inventory;

use super::ui::INVENTORY_SLOT_COUNT;

/// `game_hotspot_system` spends a gated item with `inventory.0[active_slot.0] = None`,
/// which panics if the index runs past the end. The UI creates
/// `INVENTORY_SLOT_COUNT` slots and clicking one sets the active index to it, so
/// the slot count and the default inventory have to agree.
#[test]
fn default_inventory_covers_every_slot() {
    let slots = Inventory::default().0;
    assert_eq!(
        slots.len(),
        INVENTORY_SLOT_COUNT,
        "the UI draws {INVENTORY_SLOT_COUNT} slots but the inventory holds {}, \
         so a click on the last slot would panic",
        slots.len(),
    );
}

/// Every slot the UI can select must be safe to spend, which is the same
/// statement as above but phrased as the failure.
#[test]
fn every_selectable_slot_is_spendable() {
    let slots = Inventory::default().0;
    for index in 0..INVENTORY_SLOT_COUNT {
        assert!(
            index < slots.len(),
            "slot {index} is drawn but `inventory.0[{index}]` would panic",
        );
    }
}
