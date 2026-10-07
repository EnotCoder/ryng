//! Tests for the shared press handling.
//!
//! `click_visual_sized` is a small state machine keyed on a `HashSet` of entities
//! that are currently held down. The set is what distinguishes "the pointer
//! went down over this button and came back up" (a click) from "the pointer
//! entered the button" (not a click), so the cases below pin down exactly when
//! `released` may be true.

use std::collections::HashSet;

use bevy::prelude::*;

use super::{
    BUTTON_HOVERED_SIZE, BUTTON_SIZE, ButtonSizes, PRESSED_TINT, click_visual_sized,
};

fn entity(n: u32) -> Entity {
    Entity::from_raw_u32(n).expect("entity index in range")
}

/// The visuals of a button with the standard sizes, which is what every case below
/// is about - `click_visual_sized` takes the sizes so the carousel control can
/// declare its own, and these tests are not about that.
fn standard_visual(
    interaction: &Interaction,
    held: &mut HashSet<Entity>,
    entity: Entity,
    ui_scale: f32,
) -> super::VisualState {
    click_visual_sized(interaction, held, entity, ui_scale, &ButtonSizes::default())
}

/// A click is press, then release over the same button.
#[test]
fn click_fires_once_on_release() {
    let button = entity(0);
    let mut held = HashSet::new();

    let visual = standard_visual(&Interaction::Pressed, &mut held, button, 1.0);
    assert!(!visual.released, "going down must not fire the action");
    assert!(held.contains(&button), "the button is now held");

    let visual = standard_visual(&Interaction::Hovered, &mut held, button, 1.0);
    assert!(visual.released, "coming up over the button must fire");

    let visual = standard_visual(&Interaction::Hovered, &mut held, button, 1.0);
    assert!(
        !visual.released,
        "staying hovered must not fire a second time"
    );
}

/// Releasing away from the button still completes the click, so a drag that
/// ends outside does not swallow the action.
#[test]
fn release_away_from_button_still_fires() {
    let button = entity(0);
    let mut held = HashSet::new();

    standard_visual(&Interaction::Pressed, &mut held, button, 1.0);
    let visual = standard_visual(&Interaction::None, &mut held, button, 1.0);
    assert!(visual.released, "releasing off the button must still fire");
    assert!(held.is_empty(), "the button is no longer held");
}

/// Hovering a button that was never pressed is not a click. This is the case
/// that would fire on every mouse move if `was_pressed` were not consulted.
#[test]
fn hover_without_press_does_not_fire() {
    let button = entity(0);
    let mut held = HashSet::new();

    let visual = standard_visual(&Interaction::Hovered, &mut held, button, 1.0);
    assert!(!visual.released, "hover alone must not fire");
    assert!(held.is_empty(), "hovering does not mark the button held");
}

/// Two buttons are tracked independently.
#[test]
fn buttons_do_not_interfere() {
    let a = entity(0);
    let b = entity(1);
    let mut held = HashSet::new();

    standard_visual(&Interaction::Pressed, &mut held, a, 1.0);

    let visual = standard_visual(&Interaction::Hovered, &mut held, b, 1.0);
    assert!(!visual.released, "releasing B must not fire B's hold of A");

    let visual = standard_visual(&Interaction::Hovered, &mut held, a, 1.0);
    assert!(visual.released, "A is still held and can still fire");
}

/// The set is emptied by any release, so a button cannot stay stuck down after
/// the pointer leaves the window.
#[test]
fn holding_is_cleared_on_every_exit() {
    for exit in [Interaction::Hovered, Interaction::None] {
        let button = entity(0);
        let mut held = HashSet::new();
        standard_visual(&Interaction::Pressed, &mut held, button, 1.0);
        standard_visual(&exit, &mut held, button, 1.0);
        assert!(held.is_empty(), "{exit:?} left the button marked held");
    }
}

/// Held buttons are drawn enlarged, and `ui_scale` scales that enlargement.
#[test]
fn press_visual_is_enlarged_and_scaled() {
    let button = entity(0);
    let mut held = HashSet::new();

    let pressed = standard_visual(&Interaction::Pressed, &mut held, button, 1.0);
    assert_eq!(pressed.tint, PRESSED_TINT);
    assert_eq!(pressed.size, BUTTON_HOVERED_SIZE);

    let normal = standard_visual(&Interaction::None, &mut held, button, 1.0);
    assert_eq!(normal.size, BUTTON_SIZE);

    let mut held = HashSet::new();
    let doubled = standard_visual(&Interaction::Pressed, &mut held, button, 2.0);
    assert_eq!(doubled.size, BUTTON_HOVERED_SIZE * 2.0, "scale is applied");
}
