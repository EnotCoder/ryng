use bevy::prelude::*;
use std::collections::HashSet;

use crate::acts::{default_act, Inventory, Item};
use crate::buttons;
use crate::scenes::fade::{spawn_fade_overlay, FADE_DURATION, FadePhase, RoomFade};
use crate::scenes::hotspot::{
    spawn_room, spawn_room_content, Hotspot, HotspotAction, HotspotDef, Room, RoomStory, RoomTitle,
    RoomVariant, RoomVariants, RoomVariantIndex,
};
use crate::scenes::loading::spawn_loading_overlay;
use crate::scenes::sound::TransitionSound;
use crate::state::GameState;
use crate::UiScale;

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

pub(crate) struct RoomDef {
    pub sound: TransitionSound,
    pub interactive: bool,
    pub auto_next: Option<(&'static str, f32)>,
    pub variants: Vec<RoomVariant>,
}

fn room_variant(path: &'static str, title: &'static str, story: &'static str, hotspots: Vec<HotspotDef>) -> RoomVariant {
    RoomVariant {
        path,
        title,
        story,
        hotspots,
    }
}

fn hotspot(action: HotspotAction, gate: Option<Item>) -> HotspotDef {
    HotspotDef {
        action,
        pos: Vec2::new(0.0, 0.0),
        size: Vec2::new(200.0, 300.0),
        gate,
    }
}

fn simple_room(
    path: &'static str,
    title: &'static str,
    story: &'static str,
    sound: TransitionSound,
    hotspots: Vec<HotspotDef>,
) -> RoomDef {
    RoomDef {
        sound,
        interactive: true,
        auto_next: None,
        variants: vec![room_variant(path, title, story, hotspots)],
    }
}

fn story_room(
    path: &'static str,
    title: &'static str,
    story: &'static str,
    sound: TransitionSound,
    auto_next: Option<(&'static str, f32)>,
) -> RoomDef {
    RoomDef {
        sound,
        interactive: false,
        auto_next,
        variants: vec![room_variant(path, title, story, Vec::new())],
    }
}

pub fn spawn_game_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_scale: Res<UiScale>,
    mut fade: ResMut<RoomFade>,
) {
    let s = ui_scale.0;
    *fade = RoomFade::default();

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

    commands.spawn((
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

pub(crate) fn room_def(path: &'static str) -> RoomDef {
    match path {
        "tex/rooms/floor_1/street_to_home_1.png" => {
            let next = "tex/rooms/floor_1/street_to_home_2.png";
            simple_room(
                "tex/rooms/floor_1/street_to_home_1.png",
                "Street in front of home",
                "You are tired after work and going home.\nNow you are approaching the entrance.",
                TransitionSound::NextRoom,
                vec![hotspot(HotspotAction::GoToRoom(next), None)],
            )
        }
        "tex/rooms/floor_1/street_to_home_2.png" => {
            let next = "tex/rooms/floor_1/room_concierge.png";
            simple_room(
                "tex/rooms/floor_1/street_to_home_2.png",
                "Street in front of home",
                "Enter the building by clicking on the brown door.",
                TransitionSound::NextRoom,
                vec![hotspot(HotspotAction::GoToRoom(next), None)],
            )
        }
        "tex/rooms/floor_1/room_concierge.png" => {
            let next = "tex/rooms/floor_1/room_with_elevator_floor_1.png";
            simple_room(
                "tex/rooms/floor_1/room_concierge.png",
                "Concierge",
                "Go through the concierge room,\nshowing your pass from the inventory.",
                TransitionSound::NextRoomWithOpenDoor,
                vec![hotspot(HotspotAction::GoToRoom(next), Some(Item::Pass))],
            )
        }
        "tex/rooms/floor_1/room_with_elevator_floor_1.png" | "tex/rooms/floor_1/stairs_1_floor.png" => {
            RoomDef {
                sound: TransitionSound::NextRoom,
                interactive: true,
                auto_next: None,
                variants: vec![
                    room_variant(
                        "tex/rooms/floor_1/room_with_elevator_floor_1.png",
                        "Hall - 1st floor",
                        "Choose: take the elevator or\nwalk up the stairs.",
                        vec![hotspot(
                            HotspotAction::GoToRoom("tex/rooms/elevator_Inside.png"),
                            None,
                        )],
                    ),
                    room_variant(
                        "tex/rooms/floor_1/stairs_1_floor.png",
                        "1st floor - stairs",
                        "Taking the stairs will start you from level 2.\n(The stairs are closed for now - wait for the next acts)",
                        Vec::new(),
                    ),
                ],
            }
        }
        "tex/rooms/elevator_Inside.png" => story_room(
            "tex/rooms/elevator_Inside.png",
            "Inside elevator",
            "You are inside the elevator.\nAfter 15 seconds of riding, you fall and end up in the basement.",
            TransitionSound::ElevatorFall,
            Some(("tex/rooms/basement/basement_with_elevator.png", 4.0)),
        ),
        "tex/rooms/basement/basement_with_elevator.png" => story_room(
            "tex/rooms/basement/basement_with_elevator.png",
            "Basement - elevator hall",
            "You are in the basement.\nThis is where the first act comes to an end.",
            TransitionSound::None,
            None,
        ),
        _ => story_room(
            "tex/rooms/main_fon.png",
            "Unknown room",
            "",
            TransitionSound::None,
            None,
        ),
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
    hotspots: Query<(&HotspotAction, Option<&Item>), With<Hotspot>>,
    inventory: Res<Inventory>,
    mut fade: ResMut<RoomFade>,
) {
    let mut next_room = None;
    for click in clicks.read() {
        let Ok((HotspotAction::GoToRoom(path), gate)) = hotspots.get(click.entity) else {
            continue;
        };
        let allowed = match gate {
            Some(item) => inventory.0.contains(item),
            None => true,
        };
        if allowed {
            next_room = Some(path);
        }
    }
    if let Some(path) = next_room {
        if crate::DEBUG_SHOW_HOTSPOTS {
            eprintln!("DEBUG: hotspot clicked, room -> {path}");
        }
        if matches!(&fade.phase, FadePhase::Idle) {
            fade.auto_timer = None;
            fade.pending = Some(path);
            fade.phase = FadePhase::FadeOut(Timer::from_seconds(FADE_DURATION, TimerMode::Once));
        }
    }
}

pub fn carousel_system(
    mut clicks: Query<
        (
            Entity,
            &Interaction,
            &CarouselArrow,
            Option<&mut BackgroundColor>,
            Option<&mut ImageNode>,
            Option<&mut Node>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut was_pressed: Local<HashSet<Entity>>,
    ui_scale: Res<UiScale>,
    mut rooms: Query<
        (
            Entity,
            &mut RoomVariants,
            &mut RoomVariantIndex,
            &mut RoomTitle,
            &mut RoomStory,
        ),
        With<Room>,
    >,
    mut arrow_visibility: Query<&mut Visibility, With<CarouselArrow>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let Ok((room, room_variants, mut variant_index, mut title, mut story)) = rooms.single_mut()
    else {
        return;
    };
    let variant_count = room_variants.0.len();

    for mut vis in &mut arrow_visibility {
        *vis = if variant_count > 1 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (entity, interaction, arrow, bg, img, node) in &mut clicks {
        let visual = buttons::click_visual(interaction, &mut was_pressed, entity, ui_scale.0);
        if !buttons::apply_visual(visual, bg, img, node) {
            continue;
        }
        if variant_count < 2 {
            continue;
        }
        let new_index = match arrow.0 {
            CarouselDir::Prev => (variant_index.0 + variant_count - 1) % variant_count,
            CarouselDir::Next => (variant_index.0 + 1) % variant_count,
        };
        if new_index == variant_index.0 {
            continue;
        }
        variant_index.0 = new_index;
        let variant = room_variants.0[new_index].clone();
        title.0 = variant.title;
        story.0 = variant.story;
        commands.entity(room).despawn_children();
        commands.entity(room).with_children(|parent| {
            spawn_room_content(parent, &*asset_server, &variant, true);
        });
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