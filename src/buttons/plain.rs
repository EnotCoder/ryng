use bevy::prelude::*;

use super::{NORMAL_BUTTON, spawn_button_core};

pub fn draw_button(
    parent: &mut ChildSpawnerCommands<'_>,
    text: &str,
    action: impl Component,
    ui_scale: f32,
) {
    spawn_button_core(
        parent,
        text,
        action,
        BackgroundColor(NORMAL_BUTTON),
        ui_scale,
    );
}
