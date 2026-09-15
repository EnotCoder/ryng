use bevy::prelude::*;

use super::{spawn_button_core, NORMAL_BUTTON};

pub fn draw_button(
    parent: &mut ChildSpawnerCommands<'_>,
    text: &str,
    action: impl Component,
) {
    spawn_button_core(parent, text, action, BackgroundColor(NORMAL_BUTTON));
}