//! Shared press handling for every button in the game.
//!
//! The three button systems used to each spell out the same seven-element
//! `Query` and the same `click_visual` -> `apply_visual` pair, differing only in
//! the action type and the body. They now share [`for_each_click`], so a new
//! button type only has to provide its action.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::UiScale;

use super::{apply_visual, click_visual};

/// A button carrying an action component `A`, with the visuals it may tint.
pub(crate) type ButtonQuery<'w, 's, A> = Query<
    'w,
    's,
    (
        Entity,
        &'static Interaction,
        &'static A,
        Option<&'static mut BackgroundColor>,
        Option<&'static mut ImageNode>,
        Option<&'static mut Node>,
        Option<&'static mut crate::ScaledNode>,
    ),
    (Changed<Interaction>, With<Button>),
>;

/// Applies hover/press visuals to every changed button and calls `on_click`
/// once per completed click (press, then release).
///
/// The visuals are applied whether or not this turns out to be a full click, so
/// hover states stay correct while the pointer is down.
pub(crate) fn for_each_click<A: Component>(
    mut clicks: ButtonQuery<A>,
    was_pressed: &mut HashSet<Entity>,
    ui_scale: Res<UiScale>,
    mut on_click: impl FnMut(&A),
) {
    for (entity, interaction, action, bg, img, node, scaled) in &mut clicks {
        let visual = click_visual(interaction, was_pressed, entity, ui_scale.0);
        if apply_visual(visual, bg, img, node, scaled, ui_scale.0) {
            on_click(action);
        }
    }
}
