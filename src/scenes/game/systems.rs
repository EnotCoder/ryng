use bevy::prelude::*;
use std::collections::HashSet;

use crate::UiScale;
use crate::acts::{Inventory, Item};
use crate::buttons;
use crate::scenes::fade::{FADE_DURATION, FadePhase, RoomFade};
use crate::scenes::game::ui::{CarouselArrow, CarouselDir, GameAction};
use crate::scenes::hotspot::{
    Hotspot, HotspotAction, Room, RoomStory, RoomTitle, RoomVariantIndex, RoomVariants,
    spawn_room_content,
};
use crate::state::GameState;

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
        (
            Entity,
            &Interaction,
            &GameAction,
            Option<&mut BackgroundColor>,
            Option<&mut ImageNode>,
            Option<&mut Node>,
        ),
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
