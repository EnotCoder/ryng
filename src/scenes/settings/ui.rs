use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::UiScale;
use crate::buttons;
use crate::{ScaledFont, ScaledNode};
use crate::scenes::settings::systems::{
    SettingsPanel, SettingsPanelAction, SettingsPanelOpen, Slider, SliderFill, SliderReadout,
    SliderThumb, SliderValue, SoundVolume,
};
use crate::state::GameState;

pub const TRACK_WIDTH: f32 = 220.0;
pub const TRACK_HEIGHT: f32 = 16.0;
pub const THUMB_SIZE: f32 = 26.0;
pub const ROW_GAP: f32 = 14.0;
pub const LABEL_SIZE: f32 = 20.0;
pub const TITLE_SIZE: f32 = 26.0;
pub const READOUT_WIDTH: f32 = 64.0;

pub const ACCENT_ON: Color = Color::srgb(0.95, 0.85, 0.35);
const LABEL_COLOR: Color = Color::WHITE;
const TRACK_COLOR: Color = Color::srgb(0.2, 0.2, 0.24);

// The panel card behind the rows.
const PANEL_ROW_GAP: f32 = 22.0;
const PANEL_PADDING: f32 = 28.0;
const PANEL_BORDER: f32 = 2.0;
const PANEL_RADIUS: f32 = 6.0;
const PANEL_BACKGROUND: Color = Color::srgb(0.0, 0.0, 0.0);
const PANEL_BORDER_COLOR: Color = Color::srgb(0.35, 0.35, 0.4);

/// The panel dims whatever is behind it, which is the menu rather than the game.
const SCRIM_ALPHA: f32 = 0.6;

/// A `Label + widget` row. Every setting has this shape, so the row layout and
/// its label live here once and each row contributes only its own widget.
fn settings_row(
    parent: &mut ChildSpawnerCommands<'_>,
    label: &str,
    s: &UiScale,
    widget: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) {
    parent
        .spawn((
            ScaledNode {
                gap: Some(ROW_GAP),                ..default()
            },
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: s.px(ROW_GAP),
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                ScaledFont(LABEL_SIZE),
                TextFont {
                    font_size: s.font(LABEL_SIZE),
                    ..default()
                },
                TextColor(LABEL_COLOR),
            ));
            widget(row);
        });
}

fn slider_row<R: SliderValue + 'static>(
    parent: &mut ChildSpawnerCommands<'_>,
    label: &str,
    s: &UiScale,
) {
    settings_row(parent, label, s, |row| {
        row.spawn((
            Slider::<R> {
                width: TRACK_WIDTH * s.0,
                thumb: THUMB_SIZE * s.0,
                design_width: TRACK_WIDTH,
                design_thumb: THUMB_SIZE,
                marker: std::marker::PhantomData,
            },
            BackgroundColor(TRACK_COLOR),
            ScaledNode::sized(TRACK_WIDTH, TRACK_HEIGHT),
            Node {
                width: s.px(TRACK_WIDTH),
                height: s.px(TRACK_HEIGHT),
                ..default()
            },
        ))
        .with_children(|track| {
            // Starting at full: the panel is rebuilt on every entry with the
            // resource defaults, and `apply_settings_system` pushes those to the
            // engine on the first frame.
            track.spawn((
                SliderFill,
                Pickable::IGNORE,
                BackgroundColor(ACCENT_ON),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));
            track.spawn((
                SliderThumb,
                Pickable::IGNORE,
                BackgroundColor(LABEL_COLOR),
                ScaledNode {
                    size: Some(Vec2::splat(THUMB_SIZE)),
                    left: Some(TRACK_WIDTH - THUMB_SIZE / 2.0),                    top: Some((TRACK_HEIGHT - THUMB_SIZE) / 2.0),                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: s.px(THUMB_SIZE),
                    height: s.px(THUMB_SIZE),
                    left: s.px(TRACK_WIDTH - THUMB_SIZE / 2.0),
                    top: s.px((TRACK_HEIGHT - THUMB_SIZE) / 2.0),
                    ..default()
                },
            ));
        });
        row.spawn((
            SliderReadout,
            ScaledNode::sized(READOUT_WIDTH, 0.0),
            Node {
                width: s.px(READOUT_WIDTH),
                ..default()
            },
            ScaledFont(LABEL_SIZE),
            Text::new("100%"),
            TextFont {
                font_size: s.font(LABEL_SIZE),
                ..default()
            },
            TextColor(ACCENT_ON),
        ));
    });
}

pub fn spawn_settings_panel(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_scale: Res<UiScale>,
    mut open: ResMut<SettingsPanelOpen>,
) {
    open.0 = false;
    let s = *ui_scale;
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, SCRIM_ALPHA)),
            Visibility::Hidden,
            GlobalZIndex(1),
            FocusPolicy::Block,
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
            SettingsPanel,
            DespawnOnExit(GameState::Menu),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ScaledNode {
                gap: Some(PANEL_ROW_GAP),
                padding: Some(PANEL_PADDING),
                border: Some(PANEL_BORDER),
                radius: Some(PANEL_RADIUS),
                ..default()
                    },
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: s.px(PANEL_ROW_GAP),
                        padding: UiRect::all(s.px(PANEL_PADDING)),
                        border: UiRect::all(s.px(PANEL_BORDER)),
                        border_radius: BorderRadius::all(s.px(PANEL_RADIUS)),
                        ..default()
                    },
                    BackgroundColor(PANEL_BACKGROUND),
                    BorderColor::all(PANEL_BORDER_COLOR),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Settings"),
                        ScaledFont(TITLE_SIZE),
                        TextFont {
                            font_size: s.font(TITLE_SIZE),
                            ..default()
                        },
                        TextColor(LABEL_COLOR),
                    ));
                    slider_row::<SoundVolume>(panel, "Volume", &s);
                    buttons::draw_button_with_red_texture(
                        panel,
                        "Close",
                        SettingsPanelAction::Close,
                        &asset_server,
                        s.0,
                    );
                });
        });
}
