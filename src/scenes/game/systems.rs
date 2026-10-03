use bevy::prelude::*;
use std::collections::HashSet;

use crate::UiScale;
use crate::acts::{Inventory, Item};
use crate::buttons;
use crate::scenes::fade::{FADE_DURATION, FadePhase, RoomFade};
use crate::scenes::game::inventory::ActiveInvSlot;
use crate::scenes::game::items::{
    ITEM_SIZE, ItemHotspot, ItemSprite, WorldItems, act, holds_all, item_position, should_show_in,
    spot_is_live, try_drop, try_take,
};
use crate::scenes::game::rooms::components::{
    Hotspot, HotspotAction, HotspotDef, HotspotIcon, Room, RoomPart, RoomStory, RoomTitle,
    RoomVariantIndex, RoomVariants,
};
use crate::scenes::game::rooms::data::RoomDef;
use crate::scenes::game::rooms::spawn::spawn_room_content;
use crate::scenes::game::ui::{CarouselArrow, CarouselDir, GameAction};
use crate::state::GameState;

/// The room drifts up and down a few pixels so a still picture is not perfectly
/// still. Amplitude is in design pixels; speed is a period in seconds, written as
/// `TAU / seconds` so it reads as "one full breath every 3.2s".
const BREATH_AMPLITUDE: f32 = 4.0;
const BREATH_PERIOD: f32 = 3.2;
const BREATH_SPEED: f32 = std::f32::consts::TAU / BREATH_PERIOD;

/// The arrow over a clickable hotspot fades in and out. Half a cycle is 1.6s, and
/// the alpha oscillates between fully transparent and fully opaque.
const ICON_BLINK_PERIOD: f32 = 1.6;
const ICON_BLINK_SPEED: f32 = std::f32::consts::TAU / ICON_BLINK_PERIOD;

/// `sin` runs -1..1 and alpha runs 0..1, so the amplitude is half and the offset
/// the other half.
const ALPHA_AMPLITUDE: f32 = 0.5;
const ALPHA_OFFSET: f32 = 0.5;

pub fn idle_breathe_system(time: Res<Time>, mut rooms: Query<&mut Transform, With<Room>>) {
    let t = time.elapsed_secs() * BREATH_SPEED;
    let offset = -t.sin() * BREATH_AMPLITUDE;
    for mut room in &mut rooms {
        room.translation.y = offset;
    }
}

pub fn blink_hotspot_icons(time: Res<Time>, mut icons: Query<&mut Sprite, With<HotspotIcon>>) {
    let phase = (time.elapsed_secs() * ICON_BLINK_SPEED).sin();
    let alpha = ALPHA_OFFSET + ALPHA_AMPLITUDE * phase;
    for mut icon in &mut icons {
        icon.color = Color::srgba(1.0, 1.0, 1.0, alpha);
    }
}

pub fn game_hotspot_system(
    mut clicks: MessageReader<Pointer<Click>>,
    hotspots: Query<(&HotspotAction, Option<&Item>, &HotspotDef), With<Hotspot>>,
    rooms: Query<&RoomDef, With<Room>>,
    mut inventory: ResMut<Inventory>,
    active_slot: Res<ActiveInvSlot>,
    mut world: ResMut<WorldItems>,
    mut fade: ResMut<RoomFade>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let here = rooms.single().ok().map(|def| def.variants[0].path);
    let mut next_room = None;
    for click in clicks.read() {
        let Ok((action, gate, def)) = hotspots.get(click.entity) else {
            continue;
        };
        let require = def.require;
        match action {
            HotspotAction::GoToRoom(path) => {
                // A door may ask for a set of tools, which are not spent, or for
                // a single item that *is* spent, as the concierge desk does.
                if !holds_all(&inventory, require) {
                    continue;
                }
                let allowed = match gate {
                    Some(item) => {
                        let selected = inventory.0.get(active_slot.0).copied().flatten();
                        if selected == Some(*item) {
                            inventory.0[active_slot.0] = None;
                            // The pass lands on the concierge's counter.
                            act(&mut commands, &asset_server, true);
                            true
                        } else {
                            false
                        }
                    }
                    None => true,
                };
                if allowed {
                    next_room = Some(path);
                }
            }
            other => handle_item_action(
                other,
                here,
                &mut world,
                &mut inventory,
                &active_slot,
                &mut commands,
                &asset_server,
            ),
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
    clicks: buttons::ButtonQuery<CarouselArrow>,
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

    buttons::for_each_click(clicks, &mut was_pressed, ui_scale, |arrow| {
        let Some(new_index) = step(variant_index.0, variant_count, arrow.0) else {
            return;
        };
        variant_index.0 = new_index;
        let variant = room_variants.0[new_index];
        title.0 = variant.title;
        story.0 = variant.story;
        commands.entity(room).despawn_children();
        commands.entity(room).with_children(|parent| {
            spawn_room_content(parent, &*asset_server, &variant, true);
        });
    });
}

pub fn game_button_system(
    clicks: buttons::ButtonQuery<GameAction>,
    mut was_pressed: Local<HashSet<Entity>>,
    mut next_state: ResMut<NextState<GameState>>,
    ui_scale: Res<UiScale>,
) {
    buttons::for_each_click(clicks, &mut was_pressed, ui_scale, |action| {
        fire_game(action, &mut next_state);
    });
}

/// Pick up or put down the teddy, if the clicked hotspot is one of those.
///
/// `here` is the room the player is standing in, so a click cannot act on a toy
/// that is resting somewhere else.
fn handle_item_action(
    action: &HotspotAction,
    here: Option<&'static str>,
    world: &mut WorldItems,
    inventory: &mut Inventory,
    active_slot: &ActiveInvSlot,
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    let Some(here) = here else {
        return;
    };
    let done = match action {
        HotspotAction::Take(item) => try_take(world, *item, here, inventory),
        HotspotAction::Drop(item) => try_drop(world, *item, here, inventory, active_slot.0),
        // Doors are handled by the caller.
        HotspotAction::GoToRoom(_) => false,
    };
    act(commands, asset_server, done);
}

/// Hides the pickup and put-down spots that do not apply right now.
///
/// `Visibility::Hidden` also takes the hotspot out of picking - the sprite
/// picking backend skips anything whose view visibility is false - so a hidden
/// spot cannot be clicked either, rather than just being invisible.
pub fn item_hotspot_visibility_system(
    world: Res<WorldItems>,
    rooms: Query<&RoomDef, With<Room>>,
    // Both queries write `Visibility`, so they must be declared disjoint. A
    // hotspot and the chevron beside it are separate entities, and Bevy cannot
    // prove they never overlap on its own.
    mut hotspots: Query<
        (Entity, &HotspotAction, &mut Visibility),
        (With<Hotspot>, With<ItemHotspot>, Without<HotspotIcon>),
    >,
    mut icons: Query<(Entity, &ChildOf, &mut Visibility), (With<HotspotIcon>, Without<Hotspot>)>,
) {
    let Ok(def) = rooms.single() else {
        return;
    };
    let here = def.variants[0].path;

    for (entity, action, mut visibility) in &mut hotspots {
        let wanted = if spot_is_live(&world, here, action) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        *visibility = wanted;
        // The chevron is a separate child and does not inherit `Visibility`, only
        // `InheritedVisibility`, so it is hidden on its own.
        for (_icon, parent, mut icon_visibility) in &mut icons {
            if parent.parent() == entity {
                *icon_visibility = wanted;
            }
        }
    }
}

/// Draws every item lying in the current room, and nothing in any other.
///
/// The sprites are children of the room, so they ride along with the breathing
/// motion and are despawned with the room on a transition. Because the carousel
/// clears a room's children when switching variants, sprites may have to be put
/// back, so this reconciles the world against the state every frame rather than
/// only on a change.
pub fn item_sprites_system(
    world: Res<WorldItems>,
    rooms: Query<(Entity, &RoomDef), With<Room>>,
    mut sprites: Query<(Entity, &ItemSprite), (With<RoomPart>, Without<Room>)>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let Ok((room, def)) = rooms.single() else {
        return;
    };
    let here = def.variants[0].path;

    for (entity, sprite) in &mut sprites {
        if !should_show_in(&world, sprite.0, here) {
            commands.entity(entity).despawn();
        }
    }

    let shown: Vec<Item> = sprites.iter().map(|(_, sprite)| sprite.0).collect();
    for (item, resting_room) in world.iter() {
        if resting_room != here || shown.contains(&item) {
            continue;
        }
        let Some(pos) = item_position(&def, item) else {
            continue;
        };
        let handle: Handle<Image> = asset_server.load(item.icon_path());
        commands.entity(room).with_children(|parent| {
            parent.spawn((
                RoomPart,
                ItemSprite(item),
                Sprite {
                    image: handle,
                    custom_size: Some(Vec2::splat(ITEM_SIZE)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, 0.5),
                Pickable::IGNORE,
            ));
        });
    }
}

/// The variant one step away, wrapping at both ends.
///
/// `None` when there is nowhere to go: a single variant, or an empty room.
/// Both are refused before the modulo, which would otherwise divide by zero.
pub(super) fn step(index: usize, count: usize, dir: CarouselDir) -> Option<usize> {
    if count < 2 {
        return None;
    }
    let next = match dir {
        CarouselDir::Prev => (index + count - 1) % count,
        CarouselDir::Next => (index + 1) % count,
    };
    (next != index).then_some(next)
}

fn fire_game(action: &GameAction, next: &mut NextState<GameState>) {
    match action {
        GameAction::Back => next.set(GameState::Menu),
    }
}
