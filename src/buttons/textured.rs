use bevy::prelude::*;

use super::{ButtonSizes, spawn_button_core};

pub fn draw_button_with_texture(
    parent: &mut ChildSpawnerCommands<'_>,
    text: &str,
    action: impl Component,
    asset_server: &Res<AssetServer>,
    ui_scale: f32,
) {
    spawn_button_core(
        parent,
        Some(text),
        action,
        ImageNode {
            image: asset_server.load("tex/ui/button_tex.png"),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        ui_scale,
        ButtonSizes::default(),
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
        Some(text),
        action,
        ImageNode {
            image: asset_server.load("tex/ui/button_red_tex.png"),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        ui_scale,
        ButtonSizes::default(),
    );
}

/// The carousel's control: one button, no frame, no letter, carrying a picture of
/// the shot it would take the player to.
///
/// `image` is only where the picture starts. The shot the room opens on is not known
/// to whoever spawns this, so the carousel system writes the right one as soon as it
/// reads the room - which is why the caller passes the opening shot's own preview
/// rather than a guess.
pub fn draw_picture_button(
    parent: &mut ChildSpawnerCommands<'_>,
    image: Handle<Image>,
    action: impl Component,
    sizes: ButtonSizes,
    ui_scale: f32,
) {
    spawn_button_core(
        parent,
        None,
        action,
        ImageNode {
            image,
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        ui_scale,
        sizes,
    );
}
