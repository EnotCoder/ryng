use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::prelude::*;

pub mod acts;
pub mod buttons;
pub mod cli;
pub mod scenes;
pub mod state;

use state::GameState;

pub const DEBUG_SHOW_HOTSPOTS: bool = false;

pub const DESIGN_HEIGHT: f32 = 700.0;

/// Half-width and half-height of the visible frame, in the same units as room
/// art. The camera is `FixedVertical` at `DESIGN_HEIGHT`, so this is fixed.
pub const FRAME_HALF: Vec2 = Vec2::new(640.0, 360.0);

/// How far the UI scale is allowed to move away from 1.0.
///
/// The lower bound keeps the HUD from collapsing on a very short window, where
/// `s.px` would round most constants to nothing. The upper bound stops a tall
/// window from inflating the UI past the size it was authored at - past that it
/// looks oversized rather than proportional.
const UI_SCALE_MIN: f32 = 0.4;
const UI_SCALE_MAX: f32 = 2.5;

/// Design-space to screen scale. UI code multiplies its constants by this so a
/// layout authored against `DESIGN_HEIGHT` keeps its proportions.
#[derive(Resource, Clone, Copy)]
pub struct UiScale(pub f32);

impl Default for UiScale {
    fn default() -> Self {
        Self(1.0)
    }
}

impl UiScale {
    /// A design-space length as a `Val::Px`, already scaled.
    pub fn px(&self, design: f32) -> Val {
        Val::Px(design * self.0)
    }

    /// A design-space font size, already scaled.
    pub fn font(&self, design: f32) -> FontSize {
        FontSize::Px(design * self.0)
    }
}

/// A node's geometry, in design space, so it can be recomputed on resize.
///
/// `UiScale` changes every time the window does, but a `Val::Px` baked into a
/// `Node` at spawn does not: the layout keeps the size the window had when the UI
/// was built, so shrinking the window leaves the buttons too big and enlarging it
/// leaves them too small. Only a hover put them right, and only because
/// `buttons::apply_visual` happens to read the live scale.
///
/// Recording the design-space numbers and replaying them whenever the scale changes
/// is what makes the resize visible.
///
/// **Only the fields that are set are written back.** Every field is an `Option`
/// because a bare `f32` cannot tell "authored as zero" from "not mentioned here",
/// and `..default()` fills the rest with zero - so a full-screen HUD root that only
/// wanted its `gap` scaled would have its `width: Percent(100)` overwritten with
/// `Px(0)`, and the container would collapse to nothing and take the Back button
/// with it. `None` means leave this field alone.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ScaledNode {
    /// Design-space width and height, if this node's size is fixed in pixels.
    pub size: Option<Vec2>,
    /// Design-space offsets. The axis is implied by the field name.
    pub left: Option<f32>,
    pub bottom: Option<f32>,
    pub right: Option<f32>,
    pub top: Option<f32>,
    /// Design-space gap between children in a row and in a column, if either is used.
    pub gap: Option<f32>,
    /// Design-space padding on all four sides.
    pub padding: Option<f32>,
    /// Design-space border width on all four sides.
    pub border: Option<f32>,
    /// Design-space corner radius.
    pub radius: Option<f32>,
}

impl ScaledNode {
    /// A node whose only scaled geometry is its size.
    pub fn sized(width: f32, height: f32) -> Self {
        Self {
            size: Some(Vec2::new(width, height)),
            ..default()
        }
    }
}

/// A font size in design space, for the same reason as [`ScaledNode`].
///
/// Font size lives in `TextFont` rather than in `Node`, so it cannot ride along
/// with the geometry and needs a component of its own.
#[derive(Component, Clone, Copy, Debug)]
pub struct ScaledFont(pub f32);

/// Reapplies design-space geometry after the window changed.
///
/// Guarded on `is_changed`, so it does nothing on the frames where the window is
/// still: the whole point is to react to a resize, not to run per frame.
///
/// Runs before anything that reads the layout, which in practice means before the
/// click handling - a button that is about to be pressed should already be the size
/// the pointer thinks it is.
pub fn rescale_ui_system(
    scale: Res<UiScale>,
    mut nodes: Query<(&ScaledNode, &mut Node)>,
    mut fonts: Query<(&ScaledFont, &mut TextFont)>,
) {
    if !scale.is_changed() {
        return;
    }
    let s = scale.0;

    // Each field is written only if the component mentions it. See `ScaledNode`:
    // writing all of them unconditionally is what collapsed the full-screen HUD
    // roots to `Px(0)` and pulled the Back button and the inventory into the
    // corner.
    for (design, mut node) in &mut nodes {
        if let Some(size) = design.size {
            node.width = Val::Px(size.x * s);
            node.height = Val::Px(size.y * s);
        }
        if let Some(left) = design.left {
            node.left = Val::Px(left * s);
        }
        if let Some(bottom) = design.bottom {
            node.bottom = Val::Px(bottom * s);
        }
        if let Some(right) = design.right {
            node.right = Val::Px(right * s);
        }
        if let Some(top) = design.top {
            node.top = Val::Px(top * s);
        }
        if let Some(gap) = design.gap {
            node.row_gap = Val::Px(gap * s);
            node.column_gap = Val::Px(gap * s);
        }
        if let Some(padding) = design.padding {
            node.padding = UiRect::all(Val::Px(padding * s));
        }
        if let Some(border) = design.border {
            node.border = UiRect::all(Val::Px(border * s));
        }
        if let Some(radius) = design.radius {
            node.border_radius = BorderRadius::all(Val::Px(radius * s));
        }
    }

    for (ScaledFont(design), mut font) in &mut fonts {
        font.font_size = FontSize::Px(design * s);
    }
}

#[bevy_main]
pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            scenes::loading::LoadingPlugin,
            scenes::intro::IntroPlugin,
            scenes::menu::MenuPlugin,
            scenes::settings::SettingsPlugin,
            scenes::game::GamePlugin,
        ))
        .init_state::<GameState>()
        .init_resource::<UiScale>()
        .add_systems(Startup, spawn_camera)
        // After the scale is recomputed and before anything reads the layout, so a
        // resize is applied to the UI in the same frame it happens rather than one
        // frame late.
        .add_systems(PreUpdate, (update_ui_scale, rescale_ui_system).chain())
        .add_systems(Update, scenes::loading::loading_system)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn(Camera2d)
        .insert(Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: DESIGN_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }));
}

/// Recomputes `UiScale` from the window height.
///
/// Writes only when the value actually moves. That is not a micro-optimisation: it
/// is what makes `rescale_ui_system`'s `is_changed` guard mean anything. Assigning
/// through `ResMut` marks the resource changed every frame, so an unconditional
/// assignment would make the guard always true and the rescale would rewrite the
/// UI sixty times a second - fighting `buttons::apply_visual` over the width of a
/// hovered button, which is why a hover grew and then snapped back.
fn update_ui_scale(window: Single<&Window>, mut ui_scale: ResMut<UiScale>) {
    let logical_height = window.height() / window.scale_factor();
    let wanted = (logical_height / DESIGN_HEIGHT).clamp(UI_SCALE_MIN, UI_SCALE_MAX);
    if ui_scale.0 != wanted {
        ui_scale.0 = wanted;
    }
}

#[cfg(test)]
mod ui_scale_tests;
