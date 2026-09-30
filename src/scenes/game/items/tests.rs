//! Tests for carrying and setting down the teddy.
//!
//! `Teddy` is the single source of truth: the toy is either on the floor of one
//! room or in the inventory, never both and never neither. These check the
//! state machine cannot be talked into a state it should not reach, because a
//! wrong state loses the only carryable item in the game.

use crate::acts::{Inventory, Item};

use super::{Teddy, first_free_slot, should_show_in, spot_is_live, try_drop, try_take};
use crate::scenes::game::rooms::components::HotspotAction;

const BASEMENT: &str = "tex/rooms/basement/basement_stairs_center_room.png";
const AP_1: &str = "tex/rooms/floor_2/ap_1.png";
const AP_2: &str = "tex/rooms/floor_2/ap_2.png";
const HALL: &str = "tex/rooms/floor_2/room_1.png";

fn take() -> HotspotAction {
    HotspotAction::Take(Item::Teddy)
}

fn drop() -> HotspotAction {
    HotspotAction::Drop(Item::Teddy)
}

fn with_two_free_slots() -> Inventory {
    Inventory(vec![Some(Item::Pass), None, None, Some(Item::MainKey)])
}

fn full_inventory() -> Inventory {
    Inventory(vec![
        Some(Item::Pass),
        Some(Item::MainKey),
        Some(Item::Pass),
        Some(Item::Pass),
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
    assert_eq!(first_free_slot(&full_inventory()), None);
}

#[test]
fn a_fully_empty_inventory_uses_the_first_slot() {
    assert_eq!(
        first_free_slot(&Inventory(vec![None, None, None, None])),
        Some(0)
    );
}

// ------------------------------------------------------------------ picking up

#[test]
fn taking_from_the_room_it_lies_in_succeeds() {
    let mut teddy = Teddy::Lying(BASEMENT);
    let mut inventory = with_two_free_slots();

    assert!(try_take(&mut teddy, BASEMENT, &mut inventory));
    assert_eq!(teddy, Teddy::Carried);
    assert_eq!(inventory.0[1], Some(Item::Teddy), "went into the first gap");
}

#[test]
fn taking_it_from_any_other_room_does_nothing() {
    for room in [AP_1, AP_2, "tex/rooms/floor_2/room_1.png"] {
        let mut teddy = Teddy::Lying(BASEMENT);
        let mut inventory = with_two_free_slots();
        assert!(
            !try_take(&mut teddy, room, &mut inventory),
            "took it from {room}"
        );
        assert_eq!(teddy, Teddy::Lying(BASEMENT), "state changed from {room}");
        assert_eq!(
            inventory.0,
            with_two_free_slots().0,
            "inventory changed from {room}"
        );
    }
}

#[test]
fn it_cannot_be_taken_twice() {
    let mut teddy = Teddy::Lying(BASEMENT);
    let mut inventory = with_two_free_slots();

    assert!(try_take(&mut teddy, BASEMENT, &mut inventory));
    assert!(
        !try_take(&mut teddy, BASEMENT, &mut inventory),
        "picked up twice"
    );
    assert_eq!(
        inventory
            .0
            .iter()
            .filter(|slot| **slot == Some(Item::Teddy))
            .count(),
        1,
        "the teddy is in two slots",
    );
}

/// A full inventory refuses the pickup rather than overwriting something. There
/// is no message system, so it is a silent no-op - but the toy must not vanish.
#[test]
fn a_full_inventory_refuses_the_take_and_keeps_the_teddy() {
    let mut teddy = Teddy::Lying(BASEMENT);
    let mut inventory = full_inventory();

    assert!(!try_take(&mut teddy, BASEMENT, &mut inventory));
    assert_eq!(teddy, Teddy::Lying(BASEMENT), "the teddy was lost");
    assert_eq!(inventory, full_inventory(), "a slot was overwritten");
}

// -------------------------------------------------------------- putting down

#[test]
fn dropping_where_carried_lands_the_teddy() {
    let mut teddy = Teddy::Carried;
    let mut inventory = with_two_free_slots();
    inventory.0[1] = Some(Item::Teddy);

    assert!(try_drop(&mut teddy, AP_1, &mut inventory, 1));
    assert_eq!(teddy, Teddy::Lying(AP_1));
    assert_eq!(inventory.0[1], None, "the slot it left should be empty");
}

/// The drop point only works for the slot that actually holds the toy, so
/// clicking around with another item selected does not consume it.
#[test]
fn dropping_needs_the_teddy_in_the_active_slot() {
    let mut teddy = Teddy::Carried;
    let mut inventory = with_two_free_slots();
    inventory.0[1] = Some(Item::Teddy);
    inventory.0[3] = Some(Item::MainKey);

    assert!(
        !try_drop(&mut teddy, AP_1, &mut inventory, 3),
        "dropped the wrong item"
    );
    assert_eq!(teddy, Teddy::Carried, "state changed anyway");
    assert_eq!(inventory.0[1], Some(Item::Teddy), "the teddy was consumed");
}

#[test]
fn dropping_with_nothing_selected_does_nothing() {
    let mut teddy = Teddy::Carried;
    let mut inventory = with_two_free_slots();
    inventory.0[1] = Some(Item::Teddy);

    assert!(
        !try_drop(&mut teddy, AP_1, &mut inventory, 2),
        "slot 2 is empty"
    );
    assert_eq!(teddy, Teddy::Carried);
}

#[test]
fn an_out_of_range_slot_does_not_panic() {
    let mut teddy = Teddy::Carried;
    let mut inventory = with_two_free_slots();
    inventory.0[1] = Some(Item::Teddy);

    assert!(!try_drop(&mut teddy, AP_1, &mut inventory, 99));
    assert_eq!(teddy, Teddy::Carried);
}

/// A second drop is impossible while carrying is false, so the toy can never be
/// placed in two apartments.
#[test]
fn it_cannot_be_dropped_while_it_is_lying_somewhere() {
    let mut teddy = Teddy::Lying(AP_1);
    let mut inventory = with_two_free_slots();
    inventory.0[1] = Some(Item::Teddy);

    assert!(!try_drop(&mut teddy, AP_2, &mut inventory, 1));
    assert_eq!(
        teddy,
        Teddy::Lying(AP_1),
        "the toy teleported to another room"
    );
}

// -------------------------------------------------------------- round trips

/// Taking it and putting it down again must land back in the inventory, not
/// leave the toy in a state where it is neither carried nor in the room.
#[test]
fn a_full_round_trip_returns_to_the_starting_state() {
    let start_inventory = with_two_free_slots();

    let mut teddy = Teddy::Lying(BASEMENT);
    let mut inventory = start_inventory.clone();
    assert!(try_take(&mut teddy, BASEMENT, &mut inventory));
    assert!(try_drop(&mut teddy, AP_2, &mut inventory, 1));

    assert_eq!(teddy, Teddy::Lying(AP_2));
    assert_eq!(
        inventory, start_inventory,
        "inventory did not return to its start"
    );
    assert!(
        !inventory.0.contains(&Some(Item::Teddy)),
        "the teddy is both on the floor and in the inventory",
    );
}

#[test]
fn it_can_be_moved_between_apartments() {
    // It starts on the floor in Apartment 1, is picked up there, and carried to
    // Apartment 2. Dropping requires it to be carried, so the trip has to start
    // with a take.
    let mut teddy = Teddy::Lying(AP_1);
    let mut inventory = with_two_free_slots();

    assert!(try_take(&mut teddy, AP_1, &mut inventory));
    assert!(try_drop(&mut teddy, AP_2, &mut inventory, 1));
    assert_eq!(teddy, Teddy::Lying(AP_2));
    assert!(!inventory.0.contains(&Some(Item::Teddy)));
}

/// The same room can be used to put it down and pick it straight back up, which
/// is what happens if the player changes their mind in an apartment.
#[test]
fn it_can_be_dropped_and_taken_in_the_same_room() {
    let mut teddy = Teddy::Carried;
    let mut inventory = with_two_free_slots();
    inventory.0[1] = Some(Item::Teddy);

    assert!(try_drop(&mut teddy, AP_1, &mut inventory, 1));
    assert!(try_take(&mut teddy, AP_1, &mut inventory));
    assert_eq!(teddy, Teddy::Carried);
    assert_eq!(inventory.0[1], Some(Item::Teddy));
}

// ------------------------------------------------------------------ drawing

#[test]
fn it_is_drawn_only_in_the_room_it_rests_in() {
    let teddy = Teddy::Lying(BASEMENT);
    assert!(should_show_in(&teddy, BASEMENT));
    assert!(!should_show_in(&teddy, AP_1));
    assert!(!should_show_in(&teddy, AP_2));
}

#[test]
fn a_carried_teddy_is_drawn_nowhere() {
    let teddy = Teddy::Carried;
    for room in [BASEMENT, AP_1, AP_2, "tex/rooms/floor_2/room_1.png"] {
        assert!(
            !should_show_in(&teddy, room),
            "drawn in {room} while carried"
        );
    }
}

/// The sprite follows the state, so the two cannot disagree about where the toy
/// is once a real round trip has happened.
#[test]
fn the_sprite_follows_the_state_through_a_take() {
    let mut teddy = Teddy::Lying(BASEMENT);
    let mut inventory = with_two_free_slots();

    assert!(should_show_in(&teddy, BASEMENT));
    assert!(try_take(&mut teddy, BASEMENT, &mut inventory));
    assert!(
        !should_show_in(&teddy, BASEMENT),
        "still drawn after being taken"
    );
}

// ------------------------------------------------------- which spots are live

/// With the teddy in hand, every put-down spot on the floor is offered.
#[test]
fn a_carried_teddy_offers_every_drop_spot() {
    let teddy = Teddy::Carried;
    for room in [BASEMENT, AP_1, AP_2, HALL] {
        assert!(
            spot_is_live(&teddy, room, &drop()),
            "no drop spot in {room}"
        );
        assert!(
            !spot_is_live(&teddy, room, &take()),
            "a pickup spot offered in {room} with nothing on the floor",
        );
    }
}

/// The request: leaving it in a flat clears the drop spot there and in the other
/// two, and leaves a pickup spot only where it now rests.
#[test]
fn leaving_it_clears_every_drop_spot() {
    let teddy = Teddy::Lying(AP_1);
    for room in [BASEMENT, AP_1, AP_2, HALL] {
        assert!(
            !spot_is_live(&teddy, room, &drop()),
            "a drop spot was left showing in {room}",
        );
    }
    assert!(
        spot_is_live(&teddy, AP_1, &take()),
        "cannot pick it back up"
    );
    assert!(
        !spot_is_live(&teddy, AP_2, &take()),
        "pickup spot in the wrong flat"
    );
    assert!(
        !spot_is_live(&teddy, BASEMENT, &take()),
        "pickup spot in the basement"
    );
}

/// Only the room it is in offers a pickup, so the other flats are inert.
#[test]
fn only_the_room_holding_it_offers_a_pickup() {
    for resting in [BASEMENT, AP_1, AP_2] {
        let teddy = Teddy::Lying(resting);
        for room in [BASEMENT, AP_1, AP_2, HALL] {
            assert_eq!(
                spot_is_live(&teddy, room, &take()),
                room == resting,
                "pickup spot wrong in {room} while it rests in {resting}",
            );
        }
    }
}

/// Doors belong to the room and are never hidden by any of this.
#[test]
fn doors_are_never_hidden() {
    let door = HotspotAction::GoToRoom(AP_1);
    for teddy in [Teddy::Carried, Teddy::Lying(AP_1), Teddy::Lying(BASEMENT)] {
        for room in [BASEMENT, AP_1, AP_2, HALL] {
            assert!(
                spot_is_live(&teddy, room, &door),
                "a door vanished in {room}"
            );
        }
    }
}

/// Every move the player can make must leave the spots consistent, so no walk
/// through the game can leave a spot showing where there is nothing to use.
#[test]
fn no_sequence_of_moves_leaves_a_stale_spot() {
    let rooms = [BASEMENT, AP_1, AP_2, HALL];
    let mut teddy = Teddy::Lying(BASEMENT);
    let mut inventory = with_two_free_slots();

    assert!(try_take(&mut teddy, BASEMENT, &mut inventory));
    for room in [AP_1, AP_2] {
        assert!(
            spot_is_live(&teddy, room, &drop()),
            "{room} before dropping"
        );

        assert!(try_drop(&mut teddy, room, &mut inventory, 1));
        for other in rooms {
            assert!(
                !spot_is_live(&teddy, other, &drop()),
                "stale drop spot in {other}"
            );
            assert_eq!(
                spot_is_live(&teddy, other, &take()),
                other == room,
                "pickup spot wrong in {other}",
            );
        }

        assert!(try_take(&mut teddy, room, &mut inventory));
        for other in rooms {
            assert!(
                !spot_is_live(&teddy, other, &take()),
                "stale pickup spot in {other}"
            );
        }
    }
}
