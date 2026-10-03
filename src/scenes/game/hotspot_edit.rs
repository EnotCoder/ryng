//! In-game overlay for placing hotspots.
//!
//! Enabled by `cargo run --features hotspot-editor`. Click a hotspot to select
//! it, nudge it with the arrow keys, resize it with `Q`/`E` or one axis at a
//! time with `A`/`D` and `W`/`S`, and copy the updated definition line to the
//! clipboard. `[` and `]` step through the rooms, `G` returns to the start of the
//! current act.
//!
//! It never writes to the source. A tool that edits your code while you are
//! holding the arrow keys is how you end up with a diff you cannot explain; the
//! overlay shows the line and puts it on the clipboard, and you paste it.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::FRAME_HALF;
use crate::acts::{CurrentAct, get_act};
use crate::scenes::fade::{FADE_DURATION, FadePhase, RoomFade};
use crate::scenes::game::rooms::components::HotspotAction;
use crate::scenes::game::rooms::components::{
    Hotspot, HotspotDef, Room, RoomPart, RoomVariantIndex,
};
use crate::scenes::game::rooms::data::{RoomDef, door_size_default, room_key_list};
use crate::state::GameState;

/// How far one arrow press moves a hotspot, and by how much with Shift held.
const NUDGE: f32 = 5.0;
const NUDGE_FINE: f32 = 1.0;

/// How much `Q` and `E` change its size.
const RESIZE: f32 = 5.0;

/// Marker colours. Unselected is barely there on purpose: the point is to see the
/// shape of the room, not to cover it.
const MARKER_IDLE: Color = Color::srgba(0.2, 0.7, 1.0, 0.10);
const MARKER_SELECTED: Color = Color::srgba(0.2, 1.0, 0.4, 0.35);

/// Marker z, above the room picture and the blinking arrow (z 1) but below the
/// hotspots themselves, which are picked at z 1 and must stay the top thing.
const MARKER_Z: f32 = 1.5;

/// Readout placement, in design space like the rest of the UI.
const READOUT_MARGIN: f32 = 14.0;
const READOUT_SIZE: f32 = 16.0;

/// A faint rectangle over every hotspot, and a bright one over the selection.
#[derive(Component)]
pub struct Marker {
    /// Index of the hotspot in the room's own list, so a rectangle can be
    /// matched back to it and reused rather than respawned every frame.
    index: usize,
    selected: bool,
}

/// The readout text.
#[derive(Component)]
pub struct Readout;

/// The system clipboard, absent when there is no display to own it.
///
/// Owned for the lifetime of the process rather than of the game state, so
/// re-entering the menu does not drop an X selection out from under the app.
#[derive(Resource, Default)]
pub struct Clipboard(Option<arboard::Clipboard>);

/// Built once at plugin setup. None on a headless machine, which is fine: the
/// overlay still works, it just cannot copy.
pub fn init_clipboard() -> Clipboard {
    Clipboard(arboard::Clipboard::new().ok())
}

/// What the player is editing. The coordinates are a working copy: nothing is
/// written back anywhere, this only decides what the overlay draws.
#[derive(Resource, Default)]
pub struct Edit {
    /// Index into the current variant's hotspot list.
    selected: Option<usize>,
    /// Where the overlay is drawing the selected hotspot.
    pos: Vec2,
    /// How big the overlay is drawing it.
    size: Vec2,
    /// The definition line shown in the readout and copied on `C`.
    line: String,
    /// Set by `C`, cleared once the line has reached the clipboard.
    copy_requested: bool,
    /// Other hotspots the selection overlaps. Worth surfacing, because the top
    /// one swallows the clicks meant for the one underneath.
    overlaps: Vec<usize>,
    /// Where we are in `room_keys`, for the readout and for stepping.
    room_index: usize,
}

/// The hotspots of the variant the player is looking at.
pub fn current_spots(room: &RoomDef, variant_index: usize) -> &[HotspotDef] {
    room.variants
        .get(variant_index)
        .map(|variant| variant.hotspots)
        .unwrap_or(&[])
}

// ---------------------------------------------------------------- pure bits

/// Move a hotspot by one step, keeping it inside the visible frame.
///
/// A hotspot centred below the frame is half off screen with no way to tell that
/// it is there at all, which is how a door once sat at `y = -400`.
pub fn nudge(pos: Vec2, axis: usize, sign: f32, fine: bool) -> Vec2 {
    let step = if fine { NUDGE_FINE } else { NUDGE };
    let mut next = pos;
    if axis == 0 {
        next.x = (next.x + sign * step).clamp(-FRAME_HALF.x, FRAME_HALF.x);
    } else {
        next.y = (next.y + sign * step).clamp(-FRAME_HALF.y, FRAME_HALF.y);
    }
    next
}

/// Grow or shrink a hotspot. Never negative, which would make it unpickable.
///
/// `axis` picks one side or both: a door frame is wide and short, and getting
/// there by growing both axes in step means the width has to be walked back down
/// one five-pixel step at a time.
pub fn resize(size: Vec2, axis: Option<usize>, sign: f32) -> Vec2 {
    match axis {
        Some(0) => Vec2::new((size.x + sign * RESIZE).max(0.0), size.y),
        Some(1) => Vec2::new(size.x, (size.y + sign * RESIZE).max(0.0)),
        _ => Vec2::new(
            (size.x + sign * RESIZE).max(0.0),
            (size.y + sign * RESIZE).max(0.0),
        ),
    }
}

/// Whether two hotspot rectangles share any area. Touching edges do not count.
pub fn overlaps(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2) -> bool {
    let (ahw, ahh) = (a_size.x / 2.0, a_size.y / 2.0);
    let (bhw, bhh) = (b_size.x / 2.0, b_size.y / 2.0);
    (a_pos.x - ahw < b_pos.x + bhw)
        && (b_pos.x - bhw < a_pos.x + ahw)
        && (a_pos.y - ahh < b_pos.y + bhh)
        && (b_pos.y - bhh < a_pos.y + ahh)
}

/// Room art is 1280x720 and the camera is `FixedVertical` at `DESIGN_HEIGHT`, so
/// the mapping is one to one with a fixed offset. Handy when measuring a door off
/// the picture by hand.
#[cfg(test)]
pub fn pixel_to_world(pixel: Vec2) -> Vec2 {
    Vec2::new(pixel.x - FRAME_HALF.x, FRAME_HALF.y - pixel.y)
}

#[cfg(test)]
pub fn world_to_pixel(world: Vec2) -> Vec2 {
    Vec2::new(world.x + FRAME_HALF.x, FRAME_HALF.y - world.y)
}

/// The definition line for a hotspot at the given place, ready to paste.
///
/// The size argument is omitted when it is the plain default, so the common case
/// stays as short as it was written by hand.
pub fn describe(def: &HotspotDef, pos: Vec2, size: Vec2) -> String {
    let size_arg = if size == door_size_default() {
        String::new()
    } else {
        format!(", Vec2::new({:.1}, {:.1})", size.x, size.y)
    };
    let (x, y) = (format!("{:.1}", pos.x), format!("{:.1}", pos.y));
    match def.action {
        HotspotAction::GoToRoom(target) => {
            if let Some(item) = def.gate {
                format!("gated!({target}, {x}, {y}, {item:?})")
            } else if !def.require.is_empty() {
                let tools = def
                    .require
                    .iter()
                    .map(|item| format!("{item:?}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("locked!({target}, {x}, {y}{size_arg}, &[{tools}])")
            } else {
                format!("hop!({target}, {x}, {y}{size_arg})")
            }
        }
        HotspotAction::Take(item) => format!("take!({item:?}, {x}, {y})"),
        HotspotAction::Drop(item) => format!("drop!({item:?}, {x}, {y})"),
    }
}

/// Which room of the table a given key is, for the readout and for stepping.
pub fn index_of_room(path: &str) -> Option<usize> {
    room_key_list().iter().position(|key| *key == path)
}

// ------------------------------------------------------------------- input

pub fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut clicks: MessageReader<Pointer<Click>>,
    rooms: Query<(&RoomDef, &RoomVariantIndex), With<Room>>,
    hotspots: Query<&HotspotDef, With<Hotspot>>,
    mut edit: ResMut<Edit>,
) {
    // No flag check: this module only exists when the feature is on, and the
    // flag now reads from that same feature. An earlier version had a `const`
    // here that defaulted to false, which silently turned the whole editor into
    // a no-op while still looking installed.
    let Ok((def, variants)) = rooms.single() else {
        return;
    };
    let spots = current_spots(def, variants.0);

    // Picking matches on the definition rather than on entity order, because the
    // player reads the numbers in table order and that is what the readout shows.
    for click in clicks.read() {
        let Ok(hotspot) = hotspots.get(click.entity) else {
            continue;
        };
        if let Some(index) = spots.iter().position(|spot| spot == hotspot) {
            edit.selected = Some(index);
            edit.pos = hotspot.pos;
            edit.size = hotspot.size;
            refresh(&mut edit, spots);
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        edit.selected = None;
        edit.line.clear();
        return;
    }
    if edit.selected.is_none() {
        return;
    }

    let fine = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let mut changed = false;
    for (key, axis, sign) in [
        (KeyCode::ArrowLeft, 0usize, -1.0),
        (KeyCode::ArrowRight, 0, 1.0),
        (KeyCode::ArrowDown, 1, -1.0),
        (KeyCode::ArrowUp, 1, 1.0),
    ] {
        if keys.just_pressed(key) {
            edit.pos = nudge(edit.pos, axis, sign, fine);
            changed = true;
        }
    }
    // `Q`/`E` still move both axes; `W`/`S` and `A`/`D` take one side each, which
    // is what a door frame needs. `W`/`S` are the height and `A`/`D` the width, so
    // the two pairs line up with the arrow keys.
    for (key, axis, sign) in [
        (KeyCode::KeyQ, None, -1.0),
        (KeyCode::KeyE, None, 1.0),
        (KeyCode::KeyW, Some(1usize), 1.0),
        (KeyCode::KeyS, Some(1), -1.0),
        (KeyCode::KeyA, Some(0usize), -1.0),
        (KeyCode::KeyD, Some(0), 1.0),
    ] {
        if keys.just_pressed(key) {
            edit.size = resize(edit.size, axis, sign);
            changed = true;
        }
    }
    if changed {
        refresh(&mut edit, spots);
    }
    if keys.just_pressed(KeyCode::KeyC) {
        edit.copy_requested = true;
    }
}

/// Recompute the copy line and the overlap list, so what is on screen is always
/// what `C` puts on the clipboard.
pub fn refresh(edit: &mut Edit, spots: &[HotspotDef]) {
    let Some(index) = edit.selected else {
        return;
    };
    let Some(def) = spots.get(index) else {
        return;
    };
    edit.overlaps = spots
        .iter()
        .enumerate()
        .filter(|(other, spot)| {
            *other != index && overlaps(edit.pos, edit.size, spot.pos, spot.size)
        })
        .map(|(other, _)| other)
        .collect();
    edit.line = describe(def, edit.pos, edit.size);
}

/// Steps between rooms, using the same fade a hotspot click does, so music, act
/// and the transition sound all behave normally.
pub fn navigate(
    keys: Res<ButtonInput<KeyCode>>,
    rooms: Query<&RoomDef, With<Room>>,
    act: Res<CurrentAct>,
    mut edit: ResMut<Edit>,
    mut fade: ResMut<RoomFade>,
) {
    let Ok(def) = rooms.single() else {
        return;
    };
    if let Some(index) = index_of_room(def.variants[0].path) {
        edit.room_index = index;
    }
    if !matches!(fade.phase, FadePhase::Idle) {
        return;
    }

    let list = room_key_list();
    if list.is_empty() {
        return;
    }
    let len = list.len() as i32;
    let target = if keys.just_pressed(KeyCode::BracketRight) {
        Some(((edit.room_index as i32 + 1) % len + len) % len)
    } else if keys.just_pressed(KeyCode::BracketLeft) {
        Some(((edit.room_index as i32 - 1) % len + len) % len)
    } else if keys.just_pressed(KeyCode::KeyG) {
        room_key_list()
            .iter()
            .position(|path| *path == get_act(act.0).start_room)
            .map(|index| index as i32)
    } else {
        None
    };
    let Some(target) = target else {
        return;
    };

    edit.selected = None;
    edit.line.clear();
    edit.room_index = target as usize;
    fade.auto_timer = None;
    fade.pending = Some(list[target as usize]);
    fade.phase = FadePhase::FadeOut(Timer::from_seconds(FADE_DURATION, TimerMode::Once));
}

pub fn clipboard_system(mut edit: ResMut<Edit>, mut clipboard: ResMut<Clipboard>) {
    if !edit.copy_requested {
        return;
    }
    edit.copy_requested = false;
    if let Some(clipboard) = clipboard.0.as_mut()
        && !edit.line.is_empty()
    {
        let _ = clipboard.set_text(edit.line.clone());
    }
}

// ----------------------------------------------------------------- drawing

/// Reconciles the rectangles against the room's hotspot list: updates the ones
/// that are still there, spawns the ones that are not, drops the rest. The
/// carousel clears a room's children when switching variants, so this has to
/// re-spawn rather than only draw once.
pub fn markers(
    edit: Res<Edit>,
    rooms: Query<(Entity, &RoomDef, &RoomVariantIndex), With<Room>>,
    mut drawn: Query<(Entity, &mut Sprite, &mut Transform, &Marker)>,
    mut commands: Commands,
) {
    let Ok((room, def, variants)) = rooms.single() else {
        return;
    };
    let spots = current_spots(def, variants.0);

    let mut live: HashMap<(usize, bool), Entity> = HashMap::new();
    for (entity, _sprite, _transform, marker) in &drawn {
        live.insert((marker.index, marker.selected), entity);
    }

    for (index, spot) in spots.iter().enumerate() {
        let is_selected = edit.selected == Some(index);
        let pos = if is_selected { edit.pos } else { spot.pos };
        let size = if is_selected { edit.size } else { spot.size };

        if let Some(entity) = live.remove(&(index, is_selected)) {
            if let Ok((_, mut sprite, mut transform, _)) = drawn.get_mut(entity) {
                sprite.custom_size = Some(size);
                transform.translation.x = pos.x;
                transform.translation.y = pos.y;
            }
            continue;
        }
        // A rectangle that was the selection and no longer is, or the other way
        // round, is the wrong thing to repaint in place.
        if let Some(stale) = live.remove(&(index, !is_selected))
            && let Ok((_, _, _, _)) = drawn.get_mut(stale)
        {
            commands.entity(stale).despawn();
        }
        commands.entity(room).with_children(|parent| {
            parent.spawn((
                RoomPart,
                Marker {
                    index,
                    selected: is_selected,
                },
                Sprite {
                    color: if is_selected {
                        MARKER_SELECTED
                    } else {
                        MARKER_IDLE
                    },
                    custom_size: Some(size),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, MARKER_Z),
                Pickable::IGNORE,
            ));
        });
    }

    for entity in live.into_values() {
        commands.entity(entity).despawn();
    }
}

pub fn readout_text(
    edit: Res<Edit>,
    rooms: Query<&RoomDef, With<Room>>,
    act: Res<CurrentAct>,
    mut text: Query<&mut Text, With<Readout>>,
) {
    let Ok(def) = rooms.single() else {
        return;
    };
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    let head = format!(
        "room {}/{}   {}   act: {}",
        edit.room_index + 1,
        room_key_list().len(),
        def.variants[0].title,
        get_act(act.0).name
    );
    let body = match edit.selected {
        Some(index) if !edit.line.is_empty() => {
            let clash = if edit.overlaps.is_empty() {
                String::new()
            } else {
                format!(
                    "   overlaps #{}",
                    edit.overlaps
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join(", #")
                )
            };
            format!("#{index}  {}{clash}   [C copies]", edit.line)
        }
        _ => "click a hotspot   arrows move (shift = 1px)   Q/E size   A/D width   W/S height   C copy   [ ] rooms   G act start"
            .to_string(),
    };
    text.0 = format!("{head}\n{body}");
}

pub fn spawn_overlay(mut commands: Commands, ui_scale: Res<crate::UiScale>) {
    let s = ui_scale.0;
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(READOUT_MARGIN * s),
            bottom: Val::Px(READOUT_MARGIN * s),
            ..default()
        },
        Text::new(""),
        TextFont {
            font_size: ui_scale.font(READOUT_SIZE),
            ..default()
        },
        TextColor(Color::WHITE),
        Readout,
        Pickable::IGNORE,
        DespawnOnExit(GameState::Game),
    ));
}

#[cfg(test)]
mod tests;
