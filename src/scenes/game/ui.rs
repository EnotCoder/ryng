use bevy::prelude::*;

use crate::{ScaledFont, ScaledNode, UiScale};
use crate::acts::{ActId, CurrentAct, default_act};
use crate::buttons;
use crate::scenes::fade::{RoomFade, spawn_fade_overlay};
use crate::scenes::game::StartRoom;
use crate::scenes::game::rooms::data::{act_of, all_paths, room_def};
use crate::scenes::game::rooms::{
    components::{Room, RoomTitle},
    spawn::spawn_room,
};
use crate::scenes::loading::spawn_loading_overlay;
use crate::state::GameState;

// The full-screen picture behind the room. Same art and scale as the menu and
// intro backdrops, so moving between scenes does not change it.
const BACKDROP_SCALE: f32 = 3.0;

/// Behind the room picture, which sits at z 0 as a child of `Room`. Only the
/// backdrop needs a layer of its own: nothing else is ever drawn at z 0 in this
/// scene, so this is the one z below zero in the game.
pub(crate) const BACKDROP_Z: f32 = -1.0;

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
    start_room: Res<StartRoom>,
) {
    // `UiScale` itself, not the bare f32, so the `px`/`font` helpers can be used;
    // `s.0` is passed on to the button helpers, which take the raw scale.
    let s = *ui_scale;
    *fade = RoomFade::default();
    // Act one, unless `--rooms` put us somewhere that belongs to another one -
    // otherwise `room_def` below would be asked about the room under the wrong
    // act, and the concierge is the one room where that changes the answer.
    current_act.0 = start_room
        .0
        .map_or(ActId::ActOne, |room| act_of(room).unwrap_or(ActId::ActOne));

    commands
        .spawn((
            ScaledNode {
                padding: Some(HUD_MARGIN),                ..default()
            },
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
            ScaledNode {
                padding: Some(HUD_MARGIN),
                gap: Some(HUD_ROW_GAP),
                ..default()
            },
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
                ScaledFont(CAPTION_SIZE),
                TextFont {
                    font_size: s.font(CAPTION_SIZE),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new(""),
                ScaledFont(ROOM_LABEL_SIZE),
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
            ScaledNode {
                bottom: Some(CAROUSEL_BOTTOM),
                gap: Some(CAROUSEL_GAP),
                ..default()
            },
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

    // `--rooms` names the room to open in; without it the game opens where the
    // current act opens, which is what it always did.
    let start_room = start_room
        .0
        .unwrap_or_else(|| default_act().start_room);
    let def = room_def(start_room, current_act.0);
    let handles: Vec<Handle<Image>> = all_paths().map(|path| asset_server.load(path)).collect();
    spawn_room(&mut commands, &asset_server, def, Vec3::ZERO);

    // The blurred backdrop behind the room. Below the room picture, which is a
    // child of `Room` and therefore draws at whatever z that child sits at - z 0.
    // Spawning this *after* the room and giving it the same z left the two tied,
    // and the tie is won by whichever was spawned last, so the backdrop went over
    // the room and the player looked at `main_fon.png` with the inventory on top.
    //
    // It was invisible before only because the menu and the intro both spawn their
    // own backdrop and die on the way out; nothing else had ever drawn a room and
    // this at the same time. `--rooms` made the path reachable directly, which is
    // where it showed up.
    commands.spawn((
        Sprite::from_image(asset_server.load("tex/main_fon.png")),
        Transform {
            translation: Vec3::new(0.0, 0.0, BACKDROP_Z),
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

// --------------------------------------------------------------- dialogue

/// Marks the dialogue box, so the system that fills it can find it again.
#[derive(Component)]
pub(crate) struct SpeechBox;

/// Dialogue sits above the carousel arrows and to the right of the inventory, so
/// it does not cover either. A character stands in the room, and what she says
/// belongs next to her rather than in the corner with the room name.
const SPEECH_WIDTH: f32 = 620.0;
const SPEECH_BOTTOM: f32 = 90.0;
const SPEECH_PADDING: f32 = 18.0;
const SPEECH_GAP: f32 = 10.0;
const SPEECH_TEXT_SIZE: f32 = 22.0;
const SPEECH_NAME_SIZE: f32 = 18.0;

/// A dark plate behind the text: the rooms are photographic and a line of light
/// text on top of one is unreadable.
const SPEECH_BACKGROUND: Color = Color::srgba(0.0, 0.0, 0.0, 0.72);
const SPEECH_NAME_COLOR: Color = Color::srgb(0.95, 0.85, 0.35);
const SPEECH_TEXT_COLOR: Color = Color::WHITE;

/// Spawned empty and shown only while somebody is talking, so the box does not sit
/// over the room with nothing in it.
pub(crate) fn spawn_speech_ui(mut commands: Commands, ui_scale: Res<UiScale>) {
    let s = *ui_scale;
    commands
        .spawn((
            ScaledNode {
                size: Some(Vec2::new(SPEECH_WIDTH, 0.0)),
                bottom: Some(SPEECH_BOTTOM),
                right: Some(SPEECH_MARGIN),
                padding: Some(SPEECH_PADDING),
                gap: Some(SPEECH_GAP),
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                width: s.px(SPEECH_WIDTH),
                bottom: s.px(SPEECH_BOTTOM),
                right: s.px(SPEECH_MARGIN),
                padding: UiRect::all(s.px(SPEECH_PADDING)),
                row_gap: s.px(SPEECH_GAP),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            BackgroundColor(SPEECH_BACKGROUND),
            SpeechBox,
            // Nothing is being said when the room is entered, so the box starts
            // hidden rather than flashing empty.
            Visibility::Hidden,
            Pickable::IGNORE,
            DespawnOnExit(GameState::Game),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                ScaledFont(SPEECH_NAME_SIZE),
                TextFont {
                    font_size: s.font(SPEECH_NAME_SIZE),
                    ..default()
                },
                TextColor(SPEECH_NAME_COLOR),
                SpeechName,
            ));
            parent.spawn((
                Text::new(""),
                ScaledFont(SPEECH_TEXT_SIZE),
                TextFont {
                    font_size: s.font(SPEECH_TEXT_SIZE),
                    ..default()
                },
                TextColor(SPEECH_TEXT_COLOR),
                SpeechLine,
            ));
        });
}

/// How far the box keeps from the right edge, clearing the carousel arrows.
const SPEECH_MARGIN: f32 = 20.0;

#[derive(Component)]
pub(crate) struct SpeechName;

#[derive(Component)]
pub(crate) struct SpeechLine;

/// Shows the dialogue box only while somebody is talking, so it does not sit over
/// the room with nothing in it.
///
/// Compared against what is already set rather than assigned every frame, so a
/// hidden box does not dirty its own visibility each frame.
pub(crate) fn update_speech_visibility(
    speech: Res<super::npc::Speech>,
    mut boxes: Query<&mut Visibility, With<SpeechBox>>,
) {
    let wanted = if speech.who.is_some() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut visible in &mut boxes {
        if *visible != wanted {
            *visible = wanted;
        }
    }
}

/// Writes the line and the speaker's name.
///
/// The two queries are declared disjoint because both write `Text` and Bevy cannot
/// prove the name and the line are never the same entity.
pub(crate) fn update_speech_text(
    speech: Res<super::npc::Speech>,
    mut names: Query<&mut Text, (With<SpeechName>, Without<SpeechLine>)>,
    mut lines: Query<&mut Text, (With<SpeechLine>, Without<SpeechName>)>,
) {
    let Some(who) = speech.who else {
        return;
    };
    let name = super::npc::display_name(who);
    for mut text in &mut names {
        if text.0 != name {
            text.0 = name.to_owned();
        }
    }
    for mut text in &mut lines {
        if text.0 != speech.line {
            text.0 = speech.line.clone();
        }
    }
}
