//! Tests for items in the world: picking up, putting down, and the door that
//! wants a set of them.
//!
//! `WorldItems` is the single source of truth for what is lying where, so an
//! item is either in one room or in the bag, never both and never neither. These
//! check the state machine cannot be talked into a state it should not reach.

use std::collections::HashSet;

use crate::acts::{Inventory, Item};
use crate::scenes::game::rooms::components::HotspotAction;

use super::{
    WorldItems, first_free_slot, holds_all, item_position, should_show_in, spot_is_live, try_drop,
    try_take,
};

const BASEMENT: &str = "tex/rooms/basement/basement_stairs_center_room.png";
const AP_1: &str = "tex/rooms/floor_2/ap_1.png";
const AP_2: &str = "tex/rooms/floor_2/ap_2.png";
const AP_3: &str = "tex/rooms/floor_2/ap_3.png";
const HALL: &str = "tex/rooms/floor_2/room_1.png";

/// The layout the game starts with, matching `GamePlugin`.
fn start() -> WorldItems {
    WorldItems::with_resting([
        (Item::Teddy, BASEMENT),
        (Item::Crowbar, AP_1),
        (Item::MetalCutters, AP_2),
        (Item::KeyDoor2, AP_3),
    ])
}

/// Two free slots, which is what the player has mid-game once the pass is gone.
fn bag() -> Inventory {
    Inventory(vec![Some(Item::Pass), None, None, Some(Item::MainKey)])
}

fn full() -> Inventory {
    Inventory(vec![
        Some(Item::Pass),
        Some(Item::MainKey),
        Some(Item::Crowbar),
        Some(Item::KeyDoor2),
    ])
}

// ------------------------------------------------------------- first_free_slot

#[test]
fn the_first_empty_slot_is_found() {
    assert_eq!(
        first_free_slot(&Inventory(vec![Some(Item::Pass), None, None])),
        Some(1)
    );
}

#[test]
fn a_full_inventory_has_no_free_slot() {
    assert_eq!(first_free_slot(&full()), None);
}

// ------------------------------------------------------------------ picking up

#[test]
fn taking_from_the_room_it_lies_in_succeeds() {
    let mut world = start();
    let mut inventory = bag();

    assert!(try_take(&mut world, Item::Teddy, BASEMENT, &mut inventory));
    assert_eq!(world.resting_in(Item::Teddy), None, "still in the world");
    assert_eq!(inventory.0[1], Some(Item::Teddy), "went into the first gap");
}

/// Every starting item is reachable in the room it starts in, and only there.
#[test]
fn every_starting_item_is_where_it_should_be() {
    for (item, room) in [
        (Item::Teddy, BASEMENT),
        (Item::Crowbar, AP_1),
        (Item::MetalCutters, AP_2),
        (Item::KeyDoor2, AP_3),
    ] {
        let mut world = start();
        let mut inventory = Inventory(vec![None, None, None, None]);
        assert!(try_take(&mut world, item, room, &mut inventory), "{item:?}");
        assert_eq!(world.resting_in(item), None);
    }
}

#[test]
fn taking_it_from_any_other_room_does_nothing() {
    for item in [Item::Teddy, Item::Crowbar, Item::KeyDoor2] {
        let mut world = start();
        let mut inventory = bag();
        for room in [BASEMENT, AP_1, AP_2, AP_3, HALL] {
            // Compare against where it was *before* the attempt, since a
            // successful take removes it from the world.
            let was_here = world.resting_in(item) == Some(room);
            let before = inventory.clone();
            let ok = try_take(&mut world, item, room, &mut inventory);
            assert_eq!(
                ok, was_here,
                "{item:?} in the wrong room {room} reported {ok}",
            );
            if !ok {
                assert_eq!(inventory, before, "inventory changed for {item:?}");
            }
        }
    }
}

#[test]
fn it_cannot_be_taken_twice() {
    let mut world = start();
    let mut inventory = bag();

    assert!(try_take(&mut world, Item::Teddy, BASEMENT, &mut inventory));
    assert!(!try_take(&mut world, Item::Teddy, BASEMENT, &mut inventory));
    let held = inventory
        .0
        .iter()
        .filter(|s| **s == Some(Item::Teddy))
        .count();
    assert_eq!(held, 1, "the teddy is in two slots");
}

/// A full inventory refuses the pickup rather than overwriting something. There
/// is no message system, so it is a silent no-op - but the item must not vanish.
#[test]
fn a_full_inventory_refuses_the_take_and_keeps_the_item() {
    for item in [Item::Teddy, Item::Crowbar, Item::KeyDoor2] {
        let room = world_room_of(item);
        let mut world = start();
        let mut inventory = full();

        assert!(!try_take(&mut world, item, room, &mut inventory));
        assert_eq!(world.resting_in(item), Some(room), "{item:?} was lost",);
        assert_eq!(inventory, full(), "a slot was overwritten for {item:?}");
    }
}

fn world_room_of(item: Item) -> &'static str {
    match item {
        Item::Teddy => BASEMENT,
        Item::Crowbar => AP_1,
        Item::MetalCutters => AP_2,
        Item::KeyDoor2 => AP_3,
        _ => HALL,
    }
}

// -------------------------------------------------------------- putting down

#[test]
fn dropping_where_carried_lands_the_item() {
    let mut world = start();
    let mut inventory = bag();
    assert!(try_take(&mut world, Item::Teddy, BASEMENT, &mut inventory));

    assert!(try_drop(&mut world, Item::Teddy, AP_1, &mut inventory, 1));
    assert_eq!(world.resting_in(Item::Teddy), Some(AP_1));
    assert_eq!(inventory.0[1], None, "the slot it left should be empty");
}

/// The drop point only works for the slot that actually holds the item, so
/// clicking around with another item selected does not consume it.
#[test]
fn dropping_needs_the_item_in_the_active_slot() {
    let mut world = start();
    let mut inventory = bag();
    assert!(try_take(&mut world, Item::Teddy, BASEMENT, &mut inventory));
    inventory.0[3] = Some(Item::MainKey);

    assert!(!try_drop(&mut world, Item::Teddy, AP_1, &mut inventory, 3));
    assert_eq!(
        world.resting_in(Item::Teddy),
        None,
        "the teddy was consumed"
    );
    assert_eq!(inventory.0[1], Some(Item::Teddy));
}

#[test]
fn an_out_of_range_slot_does_not_panic() {
    let mut world = start();
    let mut inventory = bag();
    assert!(try_take(&mut world, Item::Teddy, BASEMENT, &mut inventory));

    assert!(!try_drop(&mut world, Item::Teddy, AP_1, &mut inventory, 99));
    assert_eq!(world.resting_in(Item::Teddy), None);
}

/// An item already lying somewhere is not in hand, so it cannot also be dropped
/// - that would duplicate it into a second room.
#[test]
fn it_cannot_be_dropped_while_it_is_lying_somewhere() {
    for item in [Item::Crowbar, Item::MetalCutters, Item::KeyDoor2] {
        let room = world_room_of(item);
        let mut world = start();
        let mut inventory = bag();
        inventory.0[1] = Some(item);

        assert!(!try_drop(&mut world, item, AP_2, &mut inventory, 1));
        assert_eq!(world.resting_in(item), Some(room), "{item:?} teleported");
    }
}

// -------------------------------------------------------------- round trips

#[test]
fn a_full_round_trip_returns_to_the_starting_state() {
    let start_inventory = bag();
    let mut world = start();
    let mut inventory = start_inventory.clone();

    assert!(try_take(&mut world, Item::Teddy, BASEMENT, &mut inventory));
    assert!(try_drop(&mut world, Item::Teddy, AP_2, &mut inventory, 1));

    assert_eq!(world.resting_in(Item::Teddy), Some(AP_2));
    assert_eq!(inventory, start_inventory, "inventory did not return");
    assert!(!inventory.0.contains(&Some(Item::Teddy)));
}

#[test]
fn it_can_be_dropped_and_taken_in_the_same_room() {
    let mut world = start();
    let mut inventory = bag();
    assert!(try_take(&mut world, Item::Teddy, BASEMENT, &mut inventory));

    assert!(try_drop(&mut world, Item::Teddy, AP_1, &mut inventory, 1));
    assert!(try_take(&mut world, Item::Teddy, AP_1, &mut inventory));
    assert_eq!(world.resting_in(Item::Teddy), None);
}

// ------------------------------------------------------------------ drawing

#[test]
fn it_is_drawn_only_in_the_room_it_rests_in() {
    let world = start();
    for (item, room) in [
        (Item::Teddy, BASEMENT),
        (Item::Crowbar, AP_1),
        (Item::MetalCutters, AP_2),
        (Item::KeyDoor2, AP_3),
    ] {
        assert!(should_show_in(&world, item, room), "{item:?} in {room}");
        for elsewhere in [BASEMENT, AP_1, AP_2, AP_3, HALL] {
            if elsewhere != room {
                assert!(
                    !should_show_in(&world, item, elsewhere),
                    "{item:?} drawn in {elsewhere} too",
                );
            }
        }
    }
}

#[test]
fn a_carried_item_is_drawn_nowhere() {
    let mut world = start();
    let mut inventory = bag();
    assert!(try_take(&mut world, Item::Teddy, BASEMENT, &mut inventory));

    for room in [BASEMENT, AP_1, AP_2, AP_3, HALL] {
        assert!(
            !should_show_in(&world, Item::Teddy, room),
            "drawn in {room}"
        );
    }
}

// ------------------------------------------------------- which spots are live

#[test]
fn a_carried_teddy_offers_every_drop_spot() {
    let mut world = start();
    let mut inventory = bag();
    assert!(try_take(&mut world, Item::Teddy, BASEMENT, &mut inventory));

    for room in [BASEMENT, AP_1, AP_2, HALL] {
        assert!(
            spot_is_live(&world, room, &HotspotAction::Drop(Item::Teddy)),
            "no drop spot in {room}",
        );
        assert!(
            !spot_is_live(&world, room, &HotspotAction::Take(Item::Teddy)),
            "a pickup spot offered in {room} with nothing on the floor",
        );
    }
}

/// The request that started it: leaving the teddy in a flat clears the drop spot
/// there and in the other two, and leaves a pickup spot only where it now rests.
#[test]
fn leaving_it_clears_every_drop_spot() {
    let mut world = start();
    let mut inventory = bag();
    assert!(try_take(&mut world, Item::Teddy, BASEMENT, &mut inventory));
    assert!(try_drop(&mut world, Item::Teddy, AP_1, &mut inventory, 1));

    for room in [BASEMENT, AP_1, AP_2, HALL] {
        assert!(
            !spot_is_live(&world, room, &HotspotAction::Drop(Item::Teddy)),
            "a drop spot was left showing in {room}",
        );
    }
    assert!(spot_is_live(
        &world,
        AP_1,
        &HotspotAction::Take(Item::Teddy)
    ));
    assert!(!spot_is_live(
        &world,
        AP_2,
        &HotspotAction::Take(Item::Teddy)
    ));
}

#[test]
fn a_tool_in_hand_offers_a_drop_spot_everywhere() {
    let mut world = start();
    let mut inventory = bag();
    assert!(try_take(&mut world, Item::Crowbar, AP_1, &mut inventory));

    for room in [BASEMENT, AP_1, AP_2, HALL] {
        assert!(
            spot_is_live(&world, room, &HotspotAction::Drop(Item::Crowbar)),
            "no drop spot in {room}",
        );
    }
}

#[test]
fn doors_are_never_hidden() {
    let door = HotspotAction::GoToRoom(AP_1);
    for mut world in [start(), WorldItems::default()] {
        let _ = &mut world;
        for room in [BASEMENT, AP_1, AP_2, HALL] {
            assert!(
                spot_is_live(&world, room, &door),
                "a door vanished in {room}"
            );
        }
    }
}

// ------------------------------------------------------------- the locked door

#[test]
fn the_door_stays_shut_with_nothing_collected() {
    let inventory = bag();
    assert!(!holds_all(&inventory, &Item::DOOR_TOOLS));
}

#[test]
fn the_door_stays_shut_with_only_some_of_the_tools() {
    let collected = Item::DOOR_TOOLS.to_vec();
    for missing in 0..Item::DOOR_TOOLS.len() {
        let held: Vec<Item> = collected
            .iter()
            .copied()
            .filter(|item| *item != Item::DOOR_TOOLS[missing])
            .collect();
        let inventory = Inventory(vec![
            Some(held[0]),
            Some(held[1]),
            None,
            Some(Item::MainKey),
        ]);
        assert!(
            !holds_all(&inventory, &Item::DOOR_TOOLS),
            "the door opened while {:?} was missing",
            Item::DOOR_TOOLS[missing]
        );
    }
}

#[test]
fn the_door_opens_with_all_three() {
    let tools = Item::DOOR_TOOLS;
    let inventory = Inventory(vec![
        Some(tools[0]),
        None,
        Some(tools[1]),
        Some(Item::MainKey),
    ]);
    assert!(!holds_all(&inventory, &tools), "two of three is enough");

    let inventory = Inventory(vec![
        Some(tools[0]),
        Some(tools[1]),
        Some(tools[2]),
        Some(Item::MainKey),
    ]);
    assert!(holds_all(&inventory, &tools));
}

/// Order in the bag must not matter, and duplicates of one tool must not stand
/// in for another.
#[test]
fn order_does_not_matter_and_duplicates_do_not_count() {
    let tools = Item::DOOR_TOOLS;
    let shuffled = Inventory(vec![Some(tools[2]), Some(tools[0]), None, Some(tools[1])]);
    assert!(holds_all(&shuffled, &tools));

    let two_of_one = Inventory(vec![
        Some(tools[0]),
        Some(tools[0]),
        Some(tools[1]),
        Some(Item::MainKey),
    ]);
    assert!(
        !holds_all(&two_of_one, &tools),
        "a duplicate stood in for a tool"
    );
}

/// Checking the door must not consume anything - the tools are still needed to
/// walk back through.
#[test]
fn the_tools_are_needed_again_on_the_way_back() {
    let tools = Item::DOOR_TOOLS;
    let inventory = Inventory(vec![Some(tools[0]), Some(tools[1]), Some(tools[2]), None]);
    for _ in 0..2 {
        assert!(holds_all(&inventory, &tools));
    }
    let kept = inventory
        .0
        .iter()
        .filter(|slot| slot.is_some_and(|item| tools.contains(&item)))
        .count();
    assert_eq!(kept, 3, "checking the door consumed a tool");
}

#[test]
fn an_empty_requirement_is_always_satisfied() {
    let empty: &[Item] = &[];
    assert!(holds_all(&full(), empty));
    assert!(holds_all(&Inventory(vec![None; 4]), empty));
}

/// The sprite of a resting item is drawn at the position of that room's own
/// pickup hotspot, so the two can never drift apart. What this pins is the
/// consequence: every room that offers a put-down spot must also offer a pickup
/// spot for the same item, otherwise the item could be left somewhere its sprite
/// has no position to be drawn at.
///
/// The teddy is the only droppable item, so it is the only one to check - but the
/// check is written over the table rather than over the teddy, so a future
/// droppable item is covered without editing this test.
#[test]
fn every_drop_spot_has_a_pickup_spot_to_draw_at() {
    use crate::scenes::game::rooms::components::HotspotAction;
    use crate::scenes::game::rooms::data::{all_paths, room_def};

    let mut checked = 0;
    for path in all_paths() {
        let def = room_def(path, crate::acts::ActId::ActThree);
        let droppable: Vec<Item> = def
            .variants
            .iter()
            .flat_map(|variant| variant.hotspots.iter())
            .filter_map(|hotspot| match hotspot.action {
                HotspotAction::Drop(item) => Some(item),
                _ => None,
            })
            .collect();
        for item in droppable {
            assert!(
                item_position(&def, item).is_some(),
                "{} offers a drop spot for {item:?} but no pickup spot, so its \
                 sprite would have nowhere to be drawn",
                def.variants[0].path,
            );
            checked += 1;
        }
    }
    assert!(
        checked > 0,
        "no drop spots found at all, the check is vacuous"
    );
}

/// Every item that starts in the world has a pickup hotspot in the room it
/// starts in, for the same reason.
#[test]
fn every_starting_item_has_a_spot_to_be_drawn_at() {
    use crate::scenes::game::rooms::data::room_def;

    for (item, room) in start().iter() {
        let def = room_def(room, crate::acts::ActId::ActThree);
        assert!(
            item_position(&def, item).is_some(),
            "{item:?} starts in {room} with no pickup hotspot to draw it at",
        );
    }
}

/// The three tools are distinct and are not any of the items the player already
/// had, or a door could be opened with the pass.
#[test]
fn the_tools_are_distinct_and_new() {
    let tools = Item::DOOR_TOOLS;
    let unique: HashSet<Item> = tools.iter().copied().collect();
    assert_eq!(unique.len(), tools.len(), "the same tool is listed twice");
    for tool in tools {
        assert!(tool != Item::Pass, "the pass is one of the tools");
        assert!(tool != Item::MainKey, "the key is one of the tools");
    }
}

/// Collecting all three in the intended order works end to end, and the bag ends
/// up full - which is why the teddy has to be left somewhere first.
#[test]
fn collecting_all_three_works_and_fills_the_bag() {
    let mut world = start();
    let mut inventory = Inventory(vec![None, None, None, Some(Item::MainKey)]);

    for tool in Item::DOOR_TOOLS {
        let room = world_room_of(tool);
        assert!(try_take(&mut world, tool, room, &mut inventory), "{tool:?}");
    }
    assert!(holds_all(&inventory, &Item::DOOR_TOOLS));
    assert_eq!(first_free_slot(&inventory), None, "the bag should be full");
}

/// Picking up a tool while the teddy is still held is impossible, and the tool
/// stays on the floor rather than overwriting the toy.
#[test]
fn the_teddy_must_be_left_before_the_bag_fills() {
    let mut world = start();
    let mut inventory = Inventory(vec![None, None, None, Some(Item::MainKey)]);

    assert!(try_take(&mut world, Item::Teddy, BASEMENT, &mut inventory));
    // Two of the three fit.
    assert!(try_take(&mut world, Item::Crowbar, AP_1, &mut inventory));
    assert!(try_take(
        &mut world,
        Item::MetalCutters,
        AP_2,
        &mut inventory
    ));
    // The third does not, and the teddy is safe.
    assert!(!try_take(&mut world, Item::KeyDoor2, AP_3, &mut inventory));
    assert_eq!(world.resting_in(Item::KeyDoor2), Some(AP_3));
    assert_eq!(
        inventory
            .0
            .iter()
            .filter(|s| **s == Some(Item::Teddy))
            .count(),
        1
    );

    // Leaving the teddy frees the slot and the last tool comes in.
    assert!(try_drop(&mut world, Item::Teddy, AP_1, &mut inventory, 0));
    assert!(try_take(&mut world, Item::KeyDoor2, AP_3, &mut inventory));
    assert!(holds_all(&inventory, &Item::DOOR_TOOLS));
}
