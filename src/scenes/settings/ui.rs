use bevy::prelude::*;

use crate::UiScale;
use crate::buttons;
use crate::scenes::settings::systems::{
    ACCENT_ON, SettingsPanel, SettingsPanelAction, SettingsPanelOpen, Slider, SliderFill,
    SliderThumb, VignetteCheckbox, VolumeLabel,
};
use crate::state::GameState;

const TRACK_WIDTH: f32 = 220.0;
const TRACK_HEIGHT: f32 = 16.0;
const THUMB_SIZE: f32 = 26.0;
const CHECKBOX_SIZE: f32 = 24.0;

fn volume_row(parent: &mut ChildSpawnerCommands<'_>, s: f32) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(14.0 * s),
            ..default()
        },))
        .with_children(|row| {
            row.spawn((
                Text::new("Volume"),
                TextFont {
                    font_size: FontSize::Px(20.0 * s),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            row.spawn((
                Slider {
                    width: TRACK_WIDTH * s,
                    thumb: THUMB_SIZE * s,
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.24)),
                Node {
                    width: Val::Px(TRACK_WIDTH * s),
                    height: Val::Px(TRACK_HEIGHT * s),
                    ..default()
                },
            ))
            .with_children(|track| {
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
                    BackgroundColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(THUMB_SIZE * s),
                        height: Val::Px(THUMB_SIZE * s),
                        left: Val::Px(TRACK_WIDTH * s - THUMB_SIZE * s / 2.0),
                        top: Val::Px((TRACK_HEIGHT - THUMB_SIZE) / 2.0 * s),
                        ..default()
                    },
                ));
            });
            row.spawn((
                VolumeLabel,
                Node {
                    width: Val::Px(64.0 * s),
                    ..default()
                },
                Text::new("100%"),
                TextFont {
                    font_size: FontSize::Px(20.0 * s),
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.85, 0.35)),
            ));
        });
}

fn vignette_row(parent: &mut ChildSpawnerCommands<'_>, s: f32) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(14.0 * s),
            ..default()
        },))
        .with_children(|row| {
            row.spawn((
                Text::new("Vignette"),
                TextFont {
                    font_size: FontSize::Px(20.0 * s),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            row.spawn((
                VignetteCheckbox,
                Button,
                Interaction::default(),
                BackgroundColor(ACCENT_ON),
                Node {
                    width: Val::Px(CHECKBOX_SIZE * s),
                    height: Val::Px(CHECKBOX_SIZE * s),
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
    let s = ui_scale.0;
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
            SettingsPanel,
            DespawnOnExit(GameState::Menu),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(22.0 * s),
                        padding: UiRect::all(Val::Px(28.0 * s)),
                        border: UiRect::all(Val::Px(2.0 * s)),
                        border_radius: BorderRadius::all(Val::Px(6.0 * s)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                    BorderColor::all(Color::srgb(0.35, 0.35, 0.4)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Settings"),
                        TextFont {
                            font_size: FontSize::Px(26.0 * s),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    volume_row(panel, s);
                    vignette_row(panel, s);
                    buttons::draw_button_with_red_texture(
                        panel,
                        "Close",
                        SettingsPanelAction::Close,
                        &asset_server,
                        s,
                    );
                });
        });
}
