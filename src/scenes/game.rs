use bevy::prelude::*;
use std::collections::HashSet;

use crate::buttons;
use crate::scenes::hotspot::{spawn_room, Hotspot, HotspotAction, HotspotDef, Room};
use crate::state::GameState;

#[derive(Component)]
pub enum GameAction {
    Back,
}

pub fn spawn_game_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::End,
            align_items: AlignItems::End,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        Pickable::IGNORE,
        DespawnOnExit(GameState::Game),
    ))
    .with_children(|parent| {
        buttons::draw_button(parent, "Back", GameAction::Back);
    });

    spawn_room(
        &mut commands,
        &asset_server,
        "tex/rooms/street_to_home.png",
        &room_layout("tex/rooms/street_to_home.png"),
        Vec3::new(-250.0, 0.0, 0.0),
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
}

fn room_layout(path: &'static str) -> Vec<HotspotDef> {
    match path {
        "tex/rooms/street_to_home.png" => vec![
            HotspotDef {
                action: HotspotAction::GoToRoom("tex/rooms/room_with_elevator_floor_my.png"),
                pos: Vec2::new(0.0, 0.0),
                size: Vec2::new(200.0, 300.0),
            },
        ],
        _ => Vec::new(),
    }
}

pub fn game_hotspot_system(
    mut clicks: MessageReader<Pointer<Click>>,
    hotspots: Query<&HotspotAction, With<Hotspot>>,
    rooms: Query<Entity, With<Room>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut next_room = None;
    for click in clicks.read() {
        if let Ok(HotspotAction::GoToRoom(path)) = hotspots.get(click.entity) {
            next_room = Some(path);
        }
    }
    if let Some(path) = next_room {
        if let Ok(old) = rooms.single() {
            commands.entity(old).despawn();
        }
        spawn_room(&mut commands, &asset_server, path, &room_layout(path), Vec3::new(-250.0, 0.0, 0.0));
    }
}

pub fn game_button_system(
    mut query: Query<
        (Entity, &Interaction, &GameAction, Option<&mut BackgroundColor>, Option<&mut ImageNode>),
        (Changed<Interaction>, With<Button>),
    >,
    mut was_pressed: Local<HashSet<Entity>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, interaction, action, bg, img) in &mut query {
        let visual = buttons::click_visual(interaction, &mut was_pressed, entity);
        if buttons::apply_visual(visual, bg, img) {
            fire_game(action, &mut next_state);
        }
    }
}

fn fire_game(action: &GameAction, next: &mut NextState<GameState>) {
    match action {
        GameAction::Back => next.set(GameState::Menu),
    }
}