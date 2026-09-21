use bevy::prelude::*;

use crate::UiScale;
use crate::acts::{CurrentAct, default_act};
use crate::buttons;
use crate::scenes::fade::{RoomFade, spawn_fade_overlay};
use crate::scenes::game::rooms::data::room_def;
use crate::scenes::game::rooms::{
    components::{Room, RoomTitle},
    spawn::spawn_room,
};
use crate::scenes::loading::spawn_loading_overlay;
use crate::state::GameState;

#[derive(Component)]
pub enum GameAction {
    Back,
}

#[derive(Component)]
pub struct RoomLabel;

#[derive(Component, Clone, Copy)]
pub struct CarouselArrow(pub CarouselDir);

#[derive(Component, Clone, Copy)]
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
    let s = ui_scale.0;
    *fade = RoomFade::default();
    current_act.0 = crate::acts::ActId::ActOne;

    commands
        .spawn((
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
            buttons::draw_button_with_red_texture(
                parent,
                "Back",
                GameAction::Back,
                &asset_server,
                s,
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

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::End,
                column_gap: Val::Px(80.0 * s),
                padding: UiRect {
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(25.0 * s),
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
                s,
            );
            buttons::draw_button_with_red_texture(
                parent,
                ">",
                CarouselArrow(CarouselDir::Next),
                &asset_server,
                s,
            );
        });

    let start_room = default_act().start_room;
    let def = room_def(start_room);
    let handles = vec![
        asset_server.load("tex/rooms/floor_1/street_to_home_1.png"),
        asset_server.load("tex/rooms/floor_1/street_to_home_2.png"),
        asset_server.load("tex/rooms/floor_1/room_concierge.png"),
        asset_server.load("tex/rooms/floor_1/room_with_elevator_floor_1.png"),
        asset_server.load("tex/rooms/floor_1/stairs_1_floor.png"),
        asset_server.load("tex/rooms/elevator_Inside.png"),
        asset_server.load("tex/rooms/basement/basement_with_elevator.png"),
    ];
    spawn_room(
        &mut commands,
        &asset_server,
        def.variants.clone(),
        Vec3::ZERO,
        def.interactive,
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
    spawn_loading_overlay(&mut commands, handles, s, GameState::Game);
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
