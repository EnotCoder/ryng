//! Tests for the overlay's arithmetic and its readout text.
//!
//! The overlay never writes to the source, so these are the only thing standing
//! between a player and a pasted line that does not compile.

use bevy::prelude::*;

use super::{
    RESIZE, describe, index_of_room, nudge, overlaps, pixel_to_world, resize, world_to_pixel,
};
use crate::acts::Item;
use crate::scenes::game::rooms::components::{HotspotAction, HotspotDef};
use crate::scenes::game::rooms::data::door_size_default;

fn def(action: HotspotAction, pos: Vec2, size: Vec2) -> HotspotDef {
    HotspotDef {
        action,
        pos,
        size,
        gate: None,
        require: &[],
    }
}

fn door(pos: Vec2) -> HotspotDef {
    def(
        HotspotAction::GoToRoom("tex/rooms/floor_2/ap_1.png"),
        pos,
        door_size_default(),
    )
}

// --------------------------------------------------------------------- nudge

#[test]
fn an_arrow_moves_five() {
    let moved = nudge(Vec2::new(-10.0, 20.0), 0, 1.0, false);
    assert_eq!(moved.x, -5.0);
    let moved = nudge(Vec2::new(-10.0, 20.0), 1, -1.0, false);
    assert_eq!(moved.y, 15.0);
}

/// Shift is the fine step, one pixel, because five is too coarse for lining a
/// hotspot up with a door frame.
#[test]
fn shift_moves_one() {
    assert_eq!(nudge(Vec2::ZERO, 0, 1.0, true).x, 1.0);
    assert_eq!(nudge(Vec2::ZERO, 1, 1.0, true).y, 1.0);
}

/// The whole reason this clamp exists: a hotspot centred below the frame is half
/// off screen with no way to tell it is there.
#[test]
fn a_hotspot_cannot_be_nudged_off_the_frame() {
    let mut pos = Vec2::new(0.0, -360.0);
    for _ in 0..100 {
        pos = nudge(pos, 1, -1.0, false);
    }
    assert_eq!(pos.y, -360.0, "walked off the bottom of the screen");

    let mut pos = Vec2::new(640.0, 0.0);
    for _ in 0..100 {
        pos = nudge(pos, 0, 1.0, false);
    }
    assert_eq!(pos.x, 640.0, "walked off the right of the screen");
}

#[test]
fn nudging_is_reversible_inside_the_frame() {
    let start = Vec2::new(100.0, -200.0);
    let there = nudge(start, 0, 1.0, false);
    let back = nudge(there, 0, -1.0, false);
    assert_eq!(back, start);
}

// -------------------------------------------------------------------- resize

#[test]
fn q_and_e_change_both_sides_by_five() {
    let start = Vec2::new(200.0, 300.0);
    assert_eq!(resize(start, None, 1.0), Vec2::new(205.0, 305.0));
    assert_eq!(resize(start, None, -1.0), Vec2::new(195.0, 295.0));
}

/// A door frame is wide and short. Growing both axes together and then walking
/// the width back down five pixels at a time is the tedium this axis split exists
/// to remove, so the untouched side has to come back exactly unchanged.
#[test]
fn resizing_one_axis_leaves_the_other_alone() {
    let start = Vec2::new(200.0, 100.0);
    assert_eq!(resize(start, Some(0), 1.0), Vec2::new(205.0, 100.0), "width");
    assert_eq!(resize(start, Some(0), -1.0), Vec2::new(195.0, 100.0), "width");
    assert_eq!(resize(start, Some(1), 1.0), Vec2::new(200.0, 105.0), "height");
    assert_eq!(resize(start, Some(1), -1.0), Vec2::new(200.0, 95.0), "height");
}

/// A negative rectangle would make a hotspot impossible to click.
#[test]
fn resizing_never_goes_negative() {
    let mut size = Vec2::new(10.0, 10.0);
    for _ in 0..10 {
        size = resize(size, None, -1.0);
    }
    assert_eq!(size, Vec2::ZERO);
}

/// The clamp has to hold per axis too, and it must not drag the other axis down
/// with it - a zero width with the height intact is still a clickable sliver.
#[test]
fn one_axis_hitting_zero_does_not_take_the_other_with_it() {
    let mut size = Vec2::new(5.0, 100.0);
    for _ in 0..5 {
        size = resize(size, Some(0), -1.0);
    }
    assert_eq!(size, Vec2::new(0.0, 100.0));

    let mut size = Vec2::new(100.0, 5.0);
    for _ in 0..5 {
        size = resize(size, Some(1), -1.0);
    }
    assert_eq!(size, Vec2::new(100.0, 0.0));
}

/// One axis has to be enough to reach a wide door from the default rectangle, and
/// it has to land on the size written down in the table.
#[test]
fn widening_one_axis_reaches_a_wide_door() {
    let start = door_size_default();
    let wide = Vec2::new(400.0, start.y);
    let steps = (wide.x - start.x) / RESIZE;
    let mut size = start;
    for _ in 0..steps as i32 {
        size = resize(size, Some(0), 1.0);
    }
    assert_eq!(size.x, wide.x);
    assert_eq!(size.y, start.y, "the height moved while widening");
}

// ------------------------------------------------------------------ overlaps

#[test]
fn overlapping_rectangles_are_reported() {
    // The black door at 400x400 in the centre and the way back at 200x180 below
    // it, which is the pair that really do collide.
    let black = (Vec2::ZERO, Vec2::new(400.0, 400.0));
    let back = (Vec2::new(0.0, -250.0), Vec2::new(200.0, 180.0));
    assert!(overlaps(black.0, black.1, back.0, back.1));
}

#[test]
fn rectangles_clear_of_each_other_do_not_overlap() {
    let a = (Vec2::new(-300.0, 0.0), Vec2::new(200.0, 300.0));
    let b = (Vec2::new(300.0, 0.0), Vec2::new(200.0, 300.0));
    assert!(!overlaps(a.0, a.1, b.0, b.1));
}

/// Sharing an edge is not sharing an area, and reporting it would be noise.
#[test]
fn touching_edges_do_not_count() {
    let a = (Vec2::ZERO, Vec2::new(100.0, 100.0));
    let b = (Vec2::new(100.0, 0.0), Vec2::new(100.0, 100.0));
    assert!(!overlaps(a.0, a.1, b.0, b.1));
}

/// `overlaps` is a plain geometric test and says yes when a rectangle is compared
/// against itself. Keeping it that way means `refresh` can ask the question about
/// every pair without special cases; excluding the selection is the caller's job
/// and it does so by index, not by coordinates - two hotspots sharing a spot are
/// both worth reporting.
#[test]
fn a_rectangle_does_overlap_itself() {
    let a = (Vec2::new(10.0, 10.0), Vec2::new(100.0, 100.0));
    assert!(overlaps(a.0, a.1, a.0, a.1));
}

// --------------------------------------------------------- pixel conversion

#[test]
fn pixel_and_world_round_trip() {
    for pixel in [
        Vec2::new(0.0, 0.0),
        Vec2::new(1280.0, 720.0),
        Vec2::new(617.0, 337.0),
        Vec2::new(122.0, 365.0),
    ] {
        assert_eq!(world_to_pixel(pixel_to_world(pixel)), pixel, "lost {pixel}");
    }
}

/// The centre of the picture is the origin.
///
/// Image coordinates count y downwards from the top, so the top of the picture is
/// positive y and the bottom negative - the opposite of the pixel convention, and
/// an easy sign to get wrong when measuring a door by hand.
#[test]
fn the_picture_centre_is_the_origin() {
    assert_eq!(pixel_to_world(Vec2::new(640.0, 360.0)), Vec2::ZERO);
    assert!(
        pixel_to_world(Vec2::new(0.0, 360.0)).x < 0.0,
        "left is negative"
    );
    assert_eq!(
        pixel_to_world(Vec2::new(640.0, 0.0)).y,
        360.0,
        "top is positive"
    );
    assert_eq!(pixel_to_world(Vec2::new(640.0, 720.0)).y, -360.0, "bottom");
}

/// A door measured off the picture by hand should land on what was written down.
#[test]
fn a_measured_door_lands_where_expected() {
    // The left door of the corridor, around x=122px in the 1280 wide picture.
    assert_eq!(pixel_to_world(Vec2::new(122.0, 365.0)).x, -518.0);
}

// ------------------------------------------------------------------ describe

#[test]
fn a_default_door_line_omits_the_size() {
    let line = describe(
        &door(Vec2::new(-10.0, 20.0)),
        Vec2::new(-10.0, 20.0),
        door_size_default(),
    );
    assert_eq!(line, "hop!(tex/rooms/floor_2/ap_1.png, -10.0, 20.0)");
}

#[test]
fn a_resized_door_line_carries_its_size() {
    let line = describe(
        &door(Vec2::ZERO),
        Vec2::new(0.0, -250.0),
        Vec2::new(200.0, 180.0),
    );
    assert_eq!(
        line,
        "hop!(tex/rooms/floor_2/ap_1.png, 0.0, -250.0, Vec2::new(200.0, 180.0))",
    );
}

#[test]
fn a_gated_door_line_keeps_its_item() {
    let mut gated = door(Vec2::ZERO);
    gated.gate = Some(Item::Pass);
    let line = describe(&gated, Vec2::new(0.0, 0.0), door_size_default());
    assert_eq!(line, "gated!(tex/rooms/floor_2/ap_1.png, 0.0, 0.0, Pass)");
}

#[test]
fn a_locked_door_line_lists_its_tools() {
    let mut locked = door(Vec2::ZERO);
    locked.require = &Item::DOOR_TOOLS;
    let line = describe(&locked, Vec2::new(0.0, 255.0), Vec2::new(400.0, 400.0));
    assert_eq!(
        line,
        "locked!(tex/rooms/floor_2/ap_1.png, 0.0, 255.0, Vec2::new(400.0, 400.0), \
         &[Crowbar, MetalCutters, KeyDoor2])",
    );
}

#[test]
fn an_item_spot_line_names_its_item() {
    let take = def(
        HotspotAction::Take(Item::Teddy),
        Vec2::ZERO,
        door_size_default(),
    );
    assert_eq!(
        describe(&take, Vec2::new(60.0, -210.0), door_size_default()),
        "take!(Teddy, 60.0, -210.0)"
    );

    let drop = def(
        HotspotAction::Drop(Item::Teddy),
        Vec2::ZERO,
        door_size_default(),
    );
    assert_eq!(
        describe(&drop, Vec2::new(-240.0, -170.0), door_size_default()),
        "drop!(Teddy, -240.0, -170.0)",
    );
}

/// An item spot is always the small rectangle, so the size is never part of its
/// line and a stale one cannot leak into the paste.
#[test]
fn item_lines_never_carry_a_size() {
    let take = def(
        HotspotAction::Take(Item::Crowbar),
        Vec2::ZERO,
        door_size_default(),
    );
    let line = describe(&take, Vec2::new(0.0, 0.0), Vec2::new(999.0, 999.0));
    assert_eq!(line, "take!(Crowbar, 0.0, 0.0)");
}

/// The line is what gets pasted, so it has to survive a round trip through the
/// coordinates it prints.
#[test]
fn printed_coordinates_have_one_decimal() {
    let line = describe(
        &door(Vec2::ZERO),
        Vec2::new(-312.44, -144.46),
        door_size_default(),
    );
    assert_eq!(line, "hop!(tex/rooms/floor_2/ap_1.png, -312.4, -144.5)");
}

// ------------------------------------------------------------- room ordering

#[test]
fn every_room_key_is_findable() {
    let first = crate::scenes::game::rooms::data::room_key_list()
        .first()
        .copied()
        .expect("there are rooms");
    assert_eq!(index_of_room(first), Some(0));
}

#[test]
fn a_carousel_variant_is_not_mistaken_for_a_room() {
    // floor_1/stairs_1_floor.png is the second variant of both halls and has no
    // row of its own, so stepping through rooms must not offer it.
    let list = crate::scenes::game::rooms::data::room_key_list();
    assert!(
        !list.contains(&"tex/rooms/floor_1/stairs_1_floor.png"),
        "a secondary variant leaked into the room list",
    );
    assert_eq!(index_of_room("tex/rooms/nowhere.png"), None);
}

// ------------------------------------------------- the editor owns the click

/// The editor must stand the game's own click handling down. Both systems read
/// the same `Pointer<Click>`, and a `MessageReader` does not consume it, so the
/// only way to stop a click walking the player through a door is to switch the
/// handling off.
#[test]
fn gameplay_is_off_whenever_the_editor_is_compiled_in() {
    assert_eq!(
        crate::scenes::game::gameplay_active(),
        !cfg!(feature = "hotspot-editor"),
        "with the editor built in, a click would walk through the hotspot \
         instead of selecting it",
    );
}

/// The condition as Bevy actually evaluates it, rather than as a bare function
/// call: a system gated on `gameplay_active` must not fire at all when the editor
/// is present.
#[test]
fn a_system_gated_on_gameplay_does_not_fire_under_the_editor() {
    use bevy::prelude::*;

    #[derive(Resource, Default)]
    struct Fired(u32);

    fn fire(mut fired: ResMut<Fired>) {
        fired.0 += 1;
    }
    // A run condition is a system itself, not a bare bool, which is why the
    // plugin gates with `run_if(gameplay_active)`. Note the direction: it must be
    // the gameplay that is gated, not the editor.
    fn gameplay_is_on() -> bool {
        crate::scenes::game::gameplay_active()
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Fired>()
        .add_systems(Update, fire.run_if(gameplay_is_on));
    app.update();

    let fired = app.world().resource::<Fired>().0;
    if cfg!(feature = "hotspot-editor") {
        assert_eq!(
            fired, 0,
            "gameplay ran a frame while the editor held the click"
        );
    } else {
        assert_eq!(fired, 1, "gating leaked into a build without the editor");
    }
}
