use bevy::prelude::*;
use std::collections::HashSet;

use crate::buttons;
use crate::scenes::fade::{spawn_fade_overlay, FADE_DURATION, FadePhase, RoomFade};
use crate::scenes::hotspot::{spawn_room, Hotspot, HotspotAction, HotspotDef, Room, RoomTitle};
use crate::state::GameState;
use crate::UiScale;

#[derive(Component)]
pub enum GameAction {
    Back,
}

#[derive(Component)]
pub struct RoomLabel;

pub(crate) struct RoomDef {
    pub title: &'static str,
    pub hotspots: Vec<HotspotDef>,
}

pub fn spawn_game_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_scale: Res<UiScale>,
) {
    let s = ui_scale.0;
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::End,
            align_items: AlignItems::End,
            padding: UiRect::all(Val::Px(10.0 * s)),
            ..default()
        },
        Pickable::IGNORE,
        DespawnOnExit(GameState::Game),
    ))
    .with_children(|parent| {
        buttons::draw_button_with_red_texture(parent, "Back", GameAction::Back, &asset_server, s);
    });

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            padding: UiRect::all(Val::Px(20.0 * s)),
            row_gap: Val::Px(8.0 * s),
            ..default()
        },
        Pickable::IGNORE,
        DespawnOnExit(GameState::Game),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("You are at"),
            TextFont {
                font_size: FontSize::Px(20.0 * s),
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        parent.spawn((
            Text::new(""),
            TextFont {
                font_size: FontSize::Px(16.0 * s),
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
            RoomLabel,
        ));
    });

    let def = room_def("tex/rooms/street_to_home.png");
    spawn_room(
        &mut commands,
        &asset_server,
        "tex/rooms/street_to_home.png",
        &def.hotspots,
        Vec3::new(0.0, 0.0, 0.0),
        def.title,
    );

    commands.spawn((
        Sprite::from_image(asset_server.load("tex/main_fon.png")),
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(3.0, 3.0, 1.0),
            ..default()
        },
        DespawnOnExit(GameState::Game),
    ));

    spawn_fade_overlay(&mut commands);
}

pub(crate) fn room_def(path: &'static str) -> RoomDef {
    match path {
        "tex/rooms/street_to_home.png" => RoomDef {
            title: "Street in front of home",
            hotspots: vec![
                HotspotDef {
                    action: HotspotAction::GoToRoom("tex/rooms/room_with_elevator_floor_1.png"),
                    pos: Vec2::new(0.0, 0.0),
                    size: Vec2::new(200.0, 300.0),
                },
            ],
        },
        "tex/rooms/door_my_home.png" => RoomDef {
            title: "Door - my home",
            hotspots: Vec::new(),
        },
        "tex/rooms/door_nighbor_home.png" => RoomDef {
            title: "Door - neighbor's apartment",
            hotspots: Vec::new(),
        },
        "tex/rooms/elevator_Inside.png" => RoomDef {
            title: "Inside elevator",
            hotspots: vec![
                HotspotDef {
                    action: HotspotAction::GoToRoom("tex/rooms/door_my_home.png"),
                    pos: Vec2::new(0.0, 0.0),
                    size: Vec2::new(200.0, 300.0),
                },
            ],
        },
        "tex/rooms/room_with_elevator_floor_1.png" => RoomDef {
            title: "Hall - 1st floor",
            hotspots: vec![
                HotspotDef {
                    action: HotspotAction::GoToRoom("tex/rooms/elevator_Inside.png"),
                    pos: Vec2::new(0.0, 0.0),
                    size: Vec2::new(200.0, 300.0),
                },
            ],
        },
        "tex/rooms/room_with_elevator_floor_my.png" => RoomDef {
            title: "Hall - my floor",
            hotspots: Vec::new(),
        },
        _ => RoomDef {
            title: "Unknown room",
            hotspots: Vec::new(),
        },
    }
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

pub fn game_hotspot_system(
    mut clicks: MessageReader<Pointer<Click>>,
    hotspots: Query<&HotspotAction, With<Hotspot>>,
    mut fade: ResMut<RoomFade>,
) {
    let mut next_room = None;
    for click in clicks.read() {
        if let Ok(HotspotAction::GoToRoom(path)) = hotspots.get(click.entity) {
            next_room = Some(path);
        }
    }
    if let Some(path) = next_room {
        if crate::DEBUG_SHOW_HOTSPOTS {
            eprintln!("DEBUG: hotspot clicked, room -> {path}");
        }
        if matches!(&fade.phase, FadePhase::Idle) {
            fade.pending = Some(path);
            fade.phase = FadePhase::FadeOut(Timer::from_seconds(FADE_DURATION, TimerMode::Once));
        }
    }
}

pub fn game_button_system(
    mut query: Query<
        (Entity, &Interaction, &GameAction, Option<&mut BackgroundColor>, Option<&mut ImageNode>, Option<&mut Node>),
        (Changed<Interaction>, With<Button>),
    >,
    mut was_pressed: Local<HashSet<Entity>>,
    mut next_state: ResMut<NextState<GameState>>,
    ui_scale: Res<UiScale>,
) {
    for (entity, interaction, action, bg, img, node) in &mut query {
        let visual = buttons::click_visual(interaction, &mut was_pressed, entity, ui_scale.0);
        if buttons::apply_visual(visual, bg, img, node) {
            fire_game(action, &mut next_state);
        }
    }
}

fn fire_game(action: &GameAction, next: &mut NextState<GameState>) {
    match action {
        GameAction::Back => next.set(GameState::Menu),
    }
}