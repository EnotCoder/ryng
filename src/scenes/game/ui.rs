use bevy::prelude::*;

use crate::UiScale;
use crate::acts::{CurrentAct, default_act};
use crate::buttons;
use crate::scenes::fade::{RoomFade, spawn_fade_overlay};
use crate::scenes::game::rooms::data::{all_paths, room_def};
use crate::scenes::game::rooms::{
    components::{Room, RoomTitle},
    spawn::spawn_room,
};
use crate::scenes::loading::spawn_loading_overlay;
use crate::state::GameState;

// The full-screen picture behind the room. Same art and scale as the menu and
// intro backdrops, so moving between scenes does not change it.
const BACKDROP_SCALE: f32 = 3.0;

// Room HUD layout, authored against `DESIGN_HEIGHT` and multiplied by `UiScale`
// so the proportions survive a resize. The caption sits top-left, the carousel
// arrows bottom-centre, the inventory bottom-left (see `inventory::ui`).
const HUD_MARGIN: f32 = 20.0;
const HUD_ROW_GAP: f32 = 8.0;
const CAPTION_SIZE: f32 = 20.0;
const ROOM_LABEL_SIZE: f32 = 16.0;

/// The room name is dimmed against the caption above it, so the caption reads as
/// the label and this as the value.
const ROOM_LABEL_ALPHA: f32 = 0.85;

const CAROUSEL_GAP: f32 = 80.0;
const CAROUSEL_BOTTOM: f32 = 25.0;

#[derive(Component)]
pub enum GameAction {
    Back,
}

#[derive(Component)]
pub struct RoomLabel;

#[derive(Component, Clone, Copy)]
pub struct CarouselArrow(pub CarouselDir);

#[derive(Component, Clone, Copy, Debug)]
pub enum CarouselDir {
    Prev,
    Next,
}

pub fn spawn_game_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_scale: Res<UiScale>,
    mut fade: ResMut<RoomFade>,
    mut current_act: ResMut<CurrentAct>,
) {
    // `UiScale` itself, not the bare f32, so the `px`/`font` helpers can be used;
    // `s.0` is passed on to the button helpers, which take the raw scale.
    let s = *ui_scale;
    *fade = RoomFade::default();
    current_act.0 = crate::acts::ActId::ActOne;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::End,
                align_items: AlignItems::End,
                padding: UiRect::all(s.px(HUD_MARGIN)),
                ..default()
            },
            Pickable::IGNORE,
            DespawnOnExit(GameState::Game),
        ))
        .with_children(|parent| {
            buttons::draw_button_with_red_texture(
                parent,
                "Back",
                GameAction::Back,
                &asset_server,
                s.0,
            );
        });

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                padding: UiRect::all(s.px(HUD_MARGIN)),
                row_gap: s.px(HUD_ROW_GAP),
                ..default()
            },
            Pickable::IGNORE,
            DespawnOnExit(GameState::Game),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("You are at"),
                TextFont {
                    font_size: s.font(CAPTION_SIZE),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: s.font(ROOM_LABEL_SIZE),
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, ROOM_LABEL_ALPHA)),
                RoomLabel,
            ));
        });

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::End,
                column_gap: s.px(CAROUSEL_GAP),
                padding: UiRect {
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: s.px(CAROUSEL_BOTTOM),
                },
                ..default()
            },
            Pickable::IGNORE,
            DespawnOnExit(GameState::Game),
        ))
        .with_children(|parent| {
            buttons::draw_button_with_red_texture(
                parent,
                "<",
                CarouselArrow(CarouselDir::Prev),
                &asset_server,
                s.0,
            );
            buttons::draw_button_with_red_texture(
                parent,
                ">",
                CarouselArrow(CarouselDir::Next),
                &asset_server,
                s.0,
            );
        });

    let start_room = default_act().start_room;
    let def = room_def(start_room, current_act.0);
    let handles: Vec<Handle<Image>> = all_paths().map(|path| asset_server.load(path)).collect();
    spawn_room(&mut commands, &asset_server, def, Vec3::ZERO);

    commands.spawn((
        Sprite::from_image(asset_server.load("tex/main_fon.png")),
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(BACKDROP_SCALE, BACKDROP_SCALE, 1.0),
            ..default()
        },
        DespawnOnExit(GameState::Game),
    ));

    spawn_fade_overlay(&mut commands);
    spawn_loading_overlay(&mut commands, handles, s.0, GameState::Game);
}

pub fn update_room_label(
    rooms: Query<&RoomTitle, With<Room>>,
    mut labels: Query<&mut Text, With<RoomLabel>>,
) {
    let Ok(title) = rooms.single() else {
        return;
    };
    for mut label in &mut labels {
        if label.0.as_str() != title.0 {
            label.0 = title.0.to_string();
        }
    }
}
