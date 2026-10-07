use bevy::prelude::*;
use std::collections::HashSet;

use crate::{ScaledFont, ScaledNode};

// Every button in the game is transparent behind its own art. There is no fill and
// no state colour here on purpose: a background colour would be painted over the
// picture a button carries, and the picture is the whole of what the button is - the
// carousel control is a photograph of where a press goes, and a blue rectangle behind
// it would be the only thing the player could not read.
const NORMAL_TINT: Color = Color::WHITE;
const HOVERED_TINT: Color = Color::srgb(0.8, 0.8, 0.8);
/// Dimmed while held, so a press reads without the button changing shape mid-click.
pub const PRESSED_TINT: Color = Color::srgb(0.5, 0.5, 0.5);

pub const BUTTON_SIZE: Vec2 = Vec2::new(150.0, 50.0);
pub const BUTTON_HOVERED_SIZE: Vec2 = Vec2::new(155.0, 55.0);

/// The two sizes a button rests at and grows to while the pointer is on it, in
/// design pixels.
///
/// A component rather than a pair of constants because a button carrying a picture
/// of its own has to say how big that picture is. The carousel control is four times
/// the area of a menu button, and the hover step is the same few pixels on top of
/// something else entirely - which is what `click_visual` reads instead of
/// [`BUTTON_SIZE`].
#[derive(Component, Clone, Copy, PartialEq)]
pub struct ButtonSizes {
    pub normal: Vec2,
    pub hovered: Vec2,
}

impl Default for ButtonSizes {
    /// The sizes every button in the game has used so far. A button that draws a
    /// picture passes its own.
    fn default() -> Self {
        Self {
            normal: BUTTON_SIZE,
            hovered: BUTTON_HOVERED_SIZE,
        }
    }
}

pub const BUTTON_GAP: f32 = 50.0;
/// The design-space size behind [`FONT_SIZE`], for the same reason `ScaledNode`
/// exists: a `Val::Px` written at spawn does not follow the window.
const DESIGN_FONT_SIZE: f32 = 20.0;

/// Derived from the design constant rather than repeating the number, so the two
/// cannot drift.
pub const FONT_SIZE: FontSize = FontSize::Px(DESIGN_FONT_SIZE);

mod click;
mod textured;

#[cfg(test)]
mod tests;

pub(crate) use click::{ButtonQuery, for_each_click};
pub use textured::{draw_button_with_red_texture, draw_button_with_texture, draw_picture_button};

pub(crate) fn spawn_button_core(
    parent: &mut ChildSpawnerCommands<'_>,
    text: Option<&str>,
    action: impl Component,
    background: impl Bundle,
    ui_scale: f32,
    sizes: ButtonSizes,
) {
    parent
        .spawn((
            Button,
            action,
            background,
            sizes,
            // The design-space size, so `rescale_ui_system` can put it back when
            // the window changes. Without it the button keeps whatever size the
            // window had at spawn, and only a hover - which goes through
            // `apply_visual` and reads the live scale - corrects it.
            ScaledNode::sized(sizes.normal.x, sizes.normal.y),
            Node {
                width: Val::Px(sizes.normal.x * ui_scale),
                height: Val::Px(sizes.normal.y * ui_scale),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Interaction::default(),
        ))
        .with_children(|parent| {
            // A button that draws a picture has nothing to letter over it, and an
            // empty text node over that picture would be a label of nothing.
            let Some(text) = text else {
                return;
            };
            parent.spawn((
                Text::new(text),
                ScaledFont(DESIGN_FONT_SIZE),
                TextFont {
                    font_size: {
                        if let FontSize::Px(size) = FONT_SIZE {
                            FontSize::Px(size * ui_scale)
                        } else {
                            FONT_SIZE
                        }
                    },
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub(crate) struct VisualState {
    tint: Color,
    size: Vec2,
    released: bool,
}

/// The visuals for a button press, hover or rest.
///
/// Sized from the button rather than from [`BUTTON_SIZE`], so a control carrying a
/// picture of its own grows by its own few pixels instead of being snapped to the
/// menu button's size the moment the pointer touches it.
pub(crate) fn click_visual_sized(
    interaction: &Interaction,
    was_pressed: &mut HashSet<Entity>,
    entity: Entity,
    ui_scale: f32,
    sizes: &ButtonSizes,
) -> VisualState {
    match *interaction {
        Interaction::Pressed => {
            was_pressed.insert(entity);
            VisualState {
                tint: PRESSED_TINT,
                size: sizes.hovered * ui_scale,
                released: false,
            }
        }
        Interaction::Hovered => {
            let released = was_pressed.remove(&entity);
            VisualState {
                tint: HOVERED_TINT,
                size: sizes.hovered * ui_scale,
                released,
            }
        }
        Interaction::None => {
            let released = was_pressed.remove(&entity);
            VisualState {
                tint: NORMAL_TINT,
                size: sizes.normal * ui_scale,
                released,
            }
        }
    }
}

/// Applies colors/tint and returns true if it was a "full click".
///
/// Also records the size it just applied into the button's `ScaledNode`, in design
/// space. That is what keeps a hover and a resize from fighting over the width:
/// `rescale_ui_system` replays whatever `ScaledNode` holds, so a button that is
/// hovered at the moment the window changes must have the *hovered* size recorded,
/// or the resize would restore the resting size and the button would shrink out
/// from under a pointer that is still on it.
pub(crate) fn apply_visual(
    visual: VisualState,
    img: Option<Mut<'_, ImageNode>>,
    node: Option<Mut<'_, Node>>,
    scaled: Option<Mut<'_, ScaledNode>>,
    ui_scale: f32,
) -> bool {
    if let Some(mut img) = img {
        img.color = visual.tint;
    }
    if let Some(mut node) = node {
        node.width = Val::Px(visual.size.x);
        node.height = Val::Px(visual.size.y);
    }
    if let Some(mut scaled) = scaled {
        // The scale is divided back out so this stays a round trip through the one
        // `click_visual` rather than a second place the sizes are written down.
        // Guarded because the scale is clamped at 0.4 rather than at zero, so the
        // division cannot blow up - but a zero would silently produce an infinity.
        scaled.size = (ui_scale > 0.0).then(|| visual.size / ui_scale);
    }
    visual.released
}
