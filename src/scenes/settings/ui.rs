use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::UiScale;
use crate::buttons;
use crate::scenes::settings::systems::{
    Checkbox, SettingsPanel, SettingsPanelAction, SettingsPanelOpen, Slider, SliderFill,
    SliderReadout, SliderThumb, SliderValue, SoundVolume, ToggleValue, VignetteSettings,
};
use crate::state::GameState;

pub const TRACK_WIDTH: f32 = 220.0;
pub const TRACK_HEIGHT: f32 = 16.0;
pub const THUMB_SIZE: f32 = 26.0;
pub const CHECKBOX_SIZE: f32 = 24.0;
pub const ROW_GAP: f32 = 14.0;
pub const LABEL_SIZE: f32 = 20.0;
pub const TITLE_SIZE: f32 = 26.0;
pub const READOUT_WIDTH: f32 = 64.0;

pub const ACCENT_ON: Color = Color::srgb(0.95, 0.85, 0.35);
pub const ACCENT_OFF: Color = Color::srgb(0.25, 0.25, 0.3);
const LABEL_COLOR: Color = Color::WHITE;
const TRACK_COLOR: Color = Color::srgb(0.2, 0.2, 0.24);

/// A `Label + widget` row. Every setting has this shape, so the row layout and
/// its label live here once and each row contributes only its own widget.
fn settings_row(
    parent: &mut ChildSpawnerCommands<'_>,
    label: &str,
    s: &UiScale,
    widget: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: s.px(ROW_GAP),
            ..default()
        },))
        .with_children(|row| {
            row.spawn((
                Text::new(label),
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
                marker: std::marker::PhantomData,
            },
            BackgroundColor(TRACK_COLOR),
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
            Node {
                width: s.px(READOUT_WIDTH),
                ..default()
            },
            Text::new("100%"),
            TextFont {
                font_size: s.font(LABEL_SIZE),
                ..default()
            },
            TextColor(ACCENT_ON),
        ));
    });
}

fn checkbox_row<T: ToggleValue + 'static>(
    parent: &mut ChildSpawnerCommands<'_>,
    label: &str,
    s: &UiScale,
) {
    settings_row(parent, label, s, |row| {
        row.spawn((
            Checkbox::<T> {
                marker: std::marker::PhantomData,
            },
            Button,
            Interaction::default(),
            BackgroundColor(ACCENT_ON),
            Node {
                width: s.px(CHECKBOX_SIZE),
                height: s.px(CHECKBOX_SIZE),
                ..default()
            },
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
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
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: s.px(22.0),
                        padding: UiRect::all(s.px(28.0)),
                        border: UiRect::all(s.px(2.0)),
                        border_radius: BorderRadius::all(s.px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                    BorderColor::all(Color::srgb(0.35, 0.35, 0.4)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Settings"),
                        TextFont {
                            font_size: s.font(TITLE_SIZE),
                            ..default()
                        },
                        TextColor(LABEL_COLOR),
                    ));
                    slider_row::<SoundVolume>(panel, "Volume", &s);
                    checkbox_row::<VignetteSettings>(panel, "Vignette", &s);
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
