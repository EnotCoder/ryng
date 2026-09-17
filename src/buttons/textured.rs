use bevy::prelude::*;

use super::spawn_button_core;

pub fn draw_button_with_texture(
    parent: &mut ChildSpawnerCommands<'_>,
    text: &str,
    action: impl Component,
    asset_server: &Res<AssetServer>,
    ui_scale: f32,
) {
    spawn_button_core(
        parent,
        text,
        action,
        ImageNode {
            image: asset_server.load("tex/ui/button_tex.png"),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        ui_scale,
    );
}

pub fn draw_button_with_red_texture(
    parent: &mut ChildSpawnerCommands<'_>,
    text: &str,
    action: impl Component,
    asset_server: &Res<AssetServer>,
    ui_scale: f32,
) {
    spawn_button_core(
        parent,
        text,
        action,
        ImageNode {
            image: asset_server.load("tex/ui/button_red_tex.png"),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        ui_scale,
    );
}