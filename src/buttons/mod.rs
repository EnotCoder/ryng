use bevy::prelude::*;
use std::collections::HashSet;

pub const NORMAL_BUTTON: Color = Color::srgb(0.2, 0.6, 0.9);
pub const HOVERED_BUTTON: Color = Color::srgb(0.4, 0.8, 0.6);
pub const PRESSED_BUTTON: Color = Color::srgb(0.9, 0.3, 0.3);

const NORMAL_TINT: Color = Color::WHITE;
const HOVERED_TINT: Color = Color::srgb(0.8, 0.8, 0.8);
const PRESSED_TINT: Color = Color::srgb(0.5, 0.5, 0.5);

pub const BUTTON_SIZE: Vec2 = Vec2::new(150.0, 50.0);
pub const BUTTON_HOVERED_SIZE: Vec2 = Vec2::new(160.0, 60.0);

pub const BUTTON_GAP: f32 = 50.0;
pub const FONT_SIZE: FontSize = FontSize::Px(20.0);

mod plain;
mod textured;

pub use plain::draw_button;
pub use textured::{draw_button_with_texture, draw_button_with_red_texture};

pub(crate) fn spawn_button_core(
    parent: &mut ChildSpawnerCommands<'_>,
    text: &str,
    action: impl Component,
    background: impl Bundle,
) {
    parent.spawn((
        Button,
        action,
        background,
        Node {
            width: Val::Px(BUTTON_SIZE.x),
            height: Val::Px(BUTTON_SIZE.y),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Interaction::default(),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new(text),
            TextFont {
                font_size: FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

pub(crate) struct VisualState {
    bg_color: Color,
    tint: Color,
    size: Vec2,
    released: bool,
}

pub(crate) fn click_visual(
    interaction: &Interaction,
    was_pressed: &mut HashSet<Entity>,
    entity: Entity,
) -> VisualState {
    match *interaction {
        Interaction::Pressed => {
            was_pressed.insert(entity);
            VisualState { bg_color: PRESSED_BUTTON, tint: PRESSED_TINT, size: BUTTON_HOVERED_SIZE, released: false }
        }
        Interaction::Hovered => {
            let released = was_pressed.remove(&entity);
            VisualState { bg_color: HOVERED_BUTTON, tint: HOVERED_TINT, size: BUTTON_HOVERED_SIZE, released }
        }
        Interaction::None => {
            let released = was_pressed.remove(&entity);
            VisualState { bg_color: NORMAL_BUTTON, tint: NORMAL_TINT, size: BUTTON_SIZE, released }
        }
    }
}

/// Применяет цвета/тинт и возвращает true, если был «полный клик».
pub(crate) fn apply_visual(
    visual: VisualState,
    bg: Option<Mut<'_, BackgroundColor>>,
    img: Option<Mut<'_, ImageNode>>,
    node: Option<Mut<'_, Node>>,
) -> bool {
    if let Some(mut bg) = bg { bg.0 = visual.bg_color; }
    if let Some(mut img) = img { img.color = visual.tint; }
    if let Some(mut node) = node {
        node.width = Val::Px(visual.size.x);
        node.height = Val::Px(visual.size.y);
    }
    visual.released
}