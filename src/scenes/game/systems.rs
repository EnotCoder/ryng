use bevy::ecs::system::SystemParam;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

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
    Hotspot, HotspotAction, HotspotDef, HotspotIcon, HotspotOutline, OUTLINE_COLOR, Room, RoomFlip,
    RoomPart, RoomStory, RoomTitle, RoomVariantIndex, RoomVariants,
};
use crate::scenes::game::rooms::data::{RoomDef, control_target};
use crate::scenes::game::rooms::spawn::spawn_room_content;
use crate::scenes::game::ui::{CarouselArrow, GameAction};
use crate::state::GameState;

/// The room drifts up and down a few pixels so a still picture is not perfectly
/// still. Amplitude is in design pixels; speed is a period in seconds, written as
/// `TAU / seconds` so it reads as "one full breath every 3.2s".
/// The single room on screen. Every room system reads it and none of them can work
/// with two: the game shows one picture at a time, and a room spawned while the
/// previous one is still fading out is the only way to get more than one.
type CurrentRoom<'w, 's> = Query<'w, 's, (Entity, &'static RoomDef), With<Room>>;

/// The room being flipped between shots: which shot it is on, what the caption says,
/// and the pan if one is playing.
///
/// One alias for both halves of the flip rather than a query each, because the two
/// systems have to agree on one room: `carousel_system` starts the pan and
/// `room_flip_system` finishes it, and two separately written queries would be free
/// to drift onto different shapes for the same room.
///
/// `&mut` on the pan even for the system that only checks it: the systems are
/// ordered and never run against the same room at once, so sharing the shape is
/// worth more than the access each one happens to need.
type FlippingRoom<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static RoomDef,
        &'static mut RoomVariants,
        &'static mut RoomVariantIndex,
        &'static mut RoomTitle,
        &'static mut RoomStory,
        Option<&'static mut RoomFlip>,
    ),
    With<Room>,
>;

/// Item sprites: the picture of a thing lying on the floor.
///
/// `Without<Room>` keeps the room's own root out of its children's list, and
/// `ItemSprite` is what marks a sprite as a picture rather than a door.
type ItemSpriteQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static ItemSprite), (With<RoomPart>, Without<Room>)>;

/// Pickup and put-down spots: the only hotspots that come and go with an item.
///
/// The chevron beside a hotspot is drawn as a separate entity, and both queries
/// below write `Visibility`, so they have to be declared disjoint - Bevy cannot
/// prove on its own that a `Hotspot` is never also a `HotspotIcon`.
type ItemHotspotQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static HotspotAction, &'static mut Visibility),
    (With<Hotspot>, With<ItemHotspot>, Without<HotspotIcon>),
>;

/// The blinking chevron drawn over a clickable hotspot, and its parent, so hiding
/// a spot can hide the arrow beside it in the same pass.
type HotspotIconQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static ChildOf, &'static mut Visibility),
    (With<HotspotIcon>, Without<Hotspot>),
>;

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

/// How long one full brighten-dim of the hover outline takes.
///
/// Slower than the icon's 1.6s blink on purpose. The icon is ambient - it says
/// "there is something here" and nobody is looking at it. The outline answers a
/// direct question, "is this the one", and something that pulses that hard while
/// the pointer rests on it is tiring rather than reassuring.
const OUTLINE_PULSE_PERIOD: f32 = 2.2;
const OUTLINE_PULSE_SPEED: f32 = std::f32::consts::TAU / OUTLINE_PULSE_PERIOD;

/// The outline's brightness while hovered, and while fading it back in.
///
/// Both bounds are well above what looks reasonable in isolation. The outline is
/// white over photographic art, so what matters is the contrast against whatever
/// is behind it: at 0.25 the bar measured 0.07 against a light door, which is
/// invisible in practice. 0.45 is the dimmest point of the pulse and it still
/// clears a white door; 0.95 is the brightest without ever reaching full white,
/// which would read as a solid object rather than as a highlight.
const OUTLINE_ALPHA_MIN: f32 = 0.45;
const OUTLINE_ALPHA_MAX: f32 = 0.95;

/// How fast the outline fades in and out, in seconds.
///
/// Not instant in either direction: appearing in a single frame is a flash, and
/// the whole point is to be calmer than the blinking icon. It also means moving the
/// pointer between two doors cross-fades rather than blinking twice.
const OUTLINE_FADE_SECONDS: f32 = 0.12;

/// The alpha an outline should be drawing at right now.
///
/// Split out because it is the one piece of this feature with a rule in it: the
/// pulse is scaled by how far the fade has got, so a fading outline dims as it goes
/// instead of pulsing at full strength while invisible.
pub(super) fn outline_alpha(elapsed: f32, fade: f32) -> f32 {
    let phase = (elapsed * OUTLINE_PULSE_SPEED).sin();
    let swing = OUTLINE_ALPHA_MIN
        + phase
            * ((OUTLINE_ALPHA_MAX - OUTLINE_ALPHA_MIN) / 2.0)
            + (OUTLINE_ALPHA_MAX - OUTLINE_ALPHA_MIN) / 2.0;
    swing * fade.clamp(0.0, 1.0)
}

/// How far each outline's fade has got, and where it was last seen.
///
/// A resource rather than a component on the bars: the fade has to survive the
/// bars being despawned and rebuilt, which happens every time the carousel flips a
/// variant, and re-deriving it per frame would make the outline snap instead of
/// fading when the player moves the pointer between two doors.
#[derive(Resource, Default)]
pub struct OutlineFades(HashMap<Entity, Fade>);

/// One outline's fade, and the phase offset it pulses on.
#[derive(Clone, Copy)]
struct Fade {
    current: f32,
    /// A fixed offset per entity, so two doors under the pointer are not lit in
    /// lockstep. Derived from the entity index rather than kept as a resource: it
    /// only has to differ between doors, not be meaningful.
    phase: f32,
}

/// Kept as a resource rather than a component on the bars, so a fade survives the
/// carousel rebuilding them; see [`OutlineFades`].
pub fn hover_outline_system(
    time: Res<Time>,
    mut fades: ResMut<OutlineFades>,
    // The hotspots, which is where `Hovered` lives: bevy_picking writes it there,
    // and it is the hotspot that is hovered, not its bars.
    hotspots: Query<(Entity, &Hovered), With<Hotspot>>,
    // The bars themselves, plus their parent, so a change to one hotspot reaches
    // all four of its bars. The entity is not needed: the parent identifies the
    // hotspot, which is what the system keys the fade on.
    mut bars: Query<(&ChildOf, &mut Sprite, &mut Visibility), With<HotspotOutline>>,
) {
    let delta = time.delta_secs();
    let step = (delta / OUTLINE_FADE_SECONDS).clamp(0.0, 1.0);

    // Drop fades for hotspots that are gone, so the map does not grow for the
    // whole session as the player walks the game.
    let live: HashSet<Entity> = hotspots.iter().map(|(entity, _)| entity).collect();
    fades.0.retain(|hotspot, _| live.contains(hotspot));

    for (hotspot, hovered) in &hotspots {
        let wanted = if hovered.get() { 1.0 } else { 0.0 };
        let fade = fades
            .0
            .entry(hotspot)
            .or_insert_with(|| Fade {
                // Starts visible only if it is already hovered, so an outline does
                // not fade up from nothing on the frame the pointer arrives.
                current: wanted,
                phase: hotspot.index().index() as f32 * 0.7,
            });

        if (fade.current - wanted).abs() > f32::EPSILON {
            fade.current += (wanted - fade.current) * step;
        } else {
            fade.current = wanted;
        }

        let visible = fade.current > 0.01;
        let alpha = outline_alpha(time.elapsed_secs() + fade.phase, fade.current);

        for (parent, mut sprite, mut visibility) in &mut bars {
            if parent.parent() != hotspot {
                continue;
            }
            *visibility = if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            sprite.color = OUTLINE_COLOR.with_alpha(alpha);
        }
    }
}

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

/// The things a hotspot click can change.
///
/// The inventory, the items on the floor and the pending transition are read
/// together because a click touches all three at once: picking something up changes
/// the bag *and* the world, and a door changes the transition. Splitting them into
/// separate parameters made the signature longer without saying anything.
#[derive(SystemParam)]
pub struct ClickOutcome<'w> {
    inventory: ResMut<'w, Inventory>,
    active_slot: Res<'w, ActiveInvSlot>,
    world: ResMut<'w, WorldItems>,
    fade: ResMut<'w, RoomFade>,
}

pub fn game_hotspot_system(
    mut clicks: MessageReader<Pointer<Click>>,
    hotspots: Query<(&HotspotAction, Option<&Item>, &HotspotDef), With<Hotspot>>,
    rooms: Query<&RoomDef, With<Room>>,
    mut outcome: ClickOutcome,
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
                if !holds_all(&outcome.inventory, require) {
                    continue;
                }
                let allowed = match gate {
                    Some(item) => {
                        let selected = outcome
                            .inventory
                            .0
                            .get(outcome.active_slot.0)
                            .copied()
                            .flatten();
                        if selected == Some(*item) {
                            outcome.inventory.0[outcome.active_slot.0] = None;
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
                &mut outcome.world,
                &mut outcome.inventory,
                &outcome.active_slot,
                &mut commands,
                &asset_server,
            ),
        }
    }
    if let Some(path) = next_room {
        if crate::DEBUG_SHOW_HOTSPOTS {
            eprintln!("DEBUG: hotspot clicked, room -> {path}");
        }
        let fade = &mut outcome.fade;
        if matches!(&fade.phase, FadePhase::Idle) {
            fade.auto_timer = None;
            fade.pending = Some(path);
            fade.phase = FadePhase::FadeOut(Timer::from_seconds(FADE_DURATION, TimerMode::Once));
        }
    }
}

/// Puts the shot the control would lead to on the control, and hides the control in a
/// room that cannot be flipped.
///
/// Its own system rather than part of the one that handles the press, because the
/// shot changes in three places - the room opening, a press that cuts, and a pan
/// landing - and only the first of those is a click. Reading the room every frame is
/// what catches the other two without each having to remember to redraw the button.
///
/// It runs after the carousel and the pan, so the picture swaps when the player
/// arrives rather than when they set off: a press starts a pan but leaves the shot
/// where it was, and the picture on the button is where the player is going.
pub fn carousel_control_system(
    rooms: Query<(&RoomVariants, &RoomVariantIndex), With<Room>>,
    mut control: Query<(&mut ImageNode, &mut Visibility), With<CarouselArrow>>,
    asset_server: Res<AssetServer>,
) {
    let Ok((variants, index)) = rooms.single() else {
        return;
    };
    let target = control_target(index.0, variants.0.len());

    for (mut image, mut visibility) in &mut control {
        *visibility = if target.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        let Some(to) = target else {
            continue;
        };
        let Some(path) = variants.0[to].preview else {
            continue;
        };
        let wanted = asset_server.load(path);
        // Compared before writing: assigning the handle every frame marks the image
        // dirty every frame, and Bevy rebuilds the sprite that goes with it.
        if image.image.id() != wanted.id() {
            image.image = wanted;
        }
    }
}

pub fn carousel_system(
    clicks: buttons::ButtonQuery<CarouselArrow>,
    mut was_pressed: Local<HashSet<Entity>>,
    ui_scale: Res<UiScale>,
    mut rooms: FlippingRoom,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let Ok((room, def, room_variants, mut variant_index, mut title, mut story, flipping)) =
        rooms.single_mut()
    else {
        return;
    };
    let variant_count = room_variants.0.len();

    buttons::for_each_click(clicks, &mut was_pressed, ui_scale, |_| {
        // A pan owns the room while it plays. Pressing mid-pan would either stack a
        // second set of frames on the first or turn the corridor around halfway, and
        // neither is something the player asked for; the control does nothing until
        // the shot it was heading for has arrived.
        if flipping.is_some() {
            return;
        }
        let Some(new_index) = control_target(variant_index.0, variant_count) else {
            return;
        };

        // With frames drawn for this room, the shot does not change yet - the pan
        // does, and `room_flip_system` finishes the job by landing on `new_index`.
        // The title and story stay on the shot being left for the same reason: a
        // caption naming a corridor the player is still walking towards is a lie
        // for most of the pan.
        if let Some(frames) = def.flip {
            commands.entity(room).despawn_children();
            commands.entity(room).with_children(|parent| {
                let first = frames[0];
                parent.spawn((RoomPart, Sprite::from_image(asset_server.load(first))));
            });
            // The art runs from the first shot towards the last, so `forward` is
            // which end of it the player set off from - not which way the button
            // points. Pressing from the last shot wraps to the first, and that is a
            // step backwards along the art.
            commands.entity(room).insert(RoomFlip::new(
                frames,
                new_index,
                new_index > variant_index.0,
            ));
            return;
        }

        variant_index.0 = new_index;
        let variant = room_variants.0[new_index];
        title.0 = variant.title;
        story.0 = variant.story;
        commands.entity(room).despawn_children();
        commands.entity(room).with_children(|parent| {
            spawn_room_content(parent, &asset_server, &variant, true);
        });
    });
}

/// Show the next frame of the pan the player started, then land on the shot.
///
/// The frames are children of the room like every other part of its picture, so
/// they are despawned with it on a transition and a flip that is interrupted by the
/// player walking out of the room takes its frames with it.
pub fn room_flip_system(
    mut rooms: FlippingRoom,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    let Ok((room, _, room_variants, mut variant_index, mut title, mut story, flipping)) =
        rooms.single_mut()
    else {
        return;
    };
    let Some(mut flip) = flipping else {
        return;
    };

    if !flip.timer.tick(time.delta()).just_finished() {
        return;
    }

    if flip.advance() {
        let frame = flip.frame();
        commands.entity(room).despawn_children();
        commands.entity(room).with_children(|parent| {
            parent.spawn((RoomPart, Sprite::from_image(asset_server.load(frame))));
        });
        return;
    }

    // Every frame has had its turn. `to` was fixed when the pan started, so this
    // cannot land somewhere the player did not ask for even if the room has changed
    // its mind about its shots in the meantime.
    let index = flip.to;
    variant_index.0 = index;
    let variant = room_variants.0[index];
    title.0 = variant.title;
    story.0 = variant.story;
    commands.entity(room).remove::<RoomFlip>();
    commands.entity(room).despawn_children();
    commands.entity(room).with_children(|parent| {
        spawn_room_content(parent, &asset_server, &variant, true);
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
    mut hotspots: ItemHotspotQuery,
    mut icons: HotspotIconQuery,
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
    rooms: CurrentRoom,
    mut sprites: ItemSpriteQuery,
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
        let Some(pos) = item_position(def, item) else {
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

fn fire_game(action: &GameAction, next: &mut NextState<GameState>) {
    match action {
        GameAction::Back => next.set(GameState::Menu),
    }
}
