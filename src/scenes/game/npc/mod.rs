//! Characters standing in rooms.
//!
//! An NPC is a sprite with a click target and a line of dialogue. Clicking her
//! puts her line on screen and remembers that you have spoken, so the next
//! conversation moves on to the next line.
//!
//! The sprite is a child of the room, which means it rides along with the room's
//! breathing motion and despawns with the room on a transition, and it has to be
//! reconciled against the table every frame rather than drawn once - a carousel
//! clears a room's children when it switches variant.
//!
//! NPCs are deliberately *not* hotspots. A hotspot is a doorway or a thing on the
//! floor: clicking one moves the player or changes the inventory. Clicking a
//! character only produces speech, so routing it through `HotspotAction` would
//! either add a variant that cannot move the player or special-case doors out of
//! the existing one. Two systems read the same `Pointer<Click>`, and a
//! `MessageReader` does not consume it, so this one ignores anything that is not
//! a character.

mod data;
#[cfg(test)]
mod tests;

/// The hotspot layer, re-exported so the test can read the real number rather than
/// a copy of it that would still pass if this one changed.
#[cfg(test)]
pub(crate) use crate::scenes::game::rooms::spawn::test_hotspot_z;

use bevy::prelude::*;

use crate::scenes::game::rooms::components::{Room, RoomPart};
use crate::scenes::game::rooms::data::RoomDef;
use crate::state::GameState;

use crate::scenes::game::ui::{spawn_speech_ui, update_speech_text, update_speech_visibility};

pub use data::NpcId;
pub use data::display_name;
use data::{npc, npcs};

/// Marks the sprite of a character, so the reconciliation can tell an NPC from an
/// item or a hotspot child of the same room.
#[derive(Component)]
pub struct NpcSprite(pub NpcId);

/// Marks the invisible clickable rectangle over a character.
#[derive(Component)]
pub struct NpcTarget(pub NpcId);

/// Either half of a character, paired with the room's other children.
///
/// Named because the filter is long enough that it appears in three signatures and
/// read as noise, and because `Without<Room>` is the part that matters: it is what
/// keeps the room's own root - which also carries `RoomPart`-adjacent state - out of
/// its own children's reconciliation.
type NpcChildren<'w, 's, A> = Query<
    'w,
    's,
    (Entity, &'static A),
    (
        With<RoomPart>,
        Without<crate::scenes::game::rooms::components::Room>,
    ),
>;

/// The line currently on screen, and who is saying it.
///
/// A resource rather than a component on the sprite, because it outlives the room:
/// walking away mid-sentence must not leave a granny talking in an empty corridor.
#[derive(Resource, Default)]
pub struct Speech {
    pub who: Option<NpcId>,
    pub line: String,
    /// The room the line was spoken in, so it can be dropped when the player leaves.
    pub room: Option<&'static str>,
}

impl Speech {
    /// Puts a line on screen.
    fn say(&mut self, id: NpcId, line: &str, room: &'static str) {
        self.who = Some(id);
        self.line = line.to_owned();
        self.room = Some(room);
    }

    fn clear(&mut self) {
        self.who = None;
        self.line.clear();
        self.room = None;
    }
}

/// How many conversations you have had with each character you have met.
///
/// Kept per NPC rather than per room, so the count survives leaving and coming
/// back. That is the only reason the second conversation can differ from the first.
#[derive(Resource, Default)]
pub struct TalkedTo(pub Vec<(NpcId, usize)>);

impl TalkedTo {
    /// How many times you have spoken with her.
    pub fn count(&self, id: NpcId) -> usize {
        self.0
            .iter()
            .find(|(seen, _)| *seen == id)
            .map_or(0, |(_, times)| *times)
    }

    /// Records a conversation and returns the new count.
    pub fn once_more(&mut self, id: NpcId) -> usize {
        let times = self.count(id) + 1;
        match self.0.iter_mut().find(|(seen, _)| *seen == id) {
            Some(entry) => entry.1 = times,
            None => self.0.push((id, times)),
        }
        times
    }

    /// The line for the next conversation.
    ///
    /// Walks the list in order and then repeats the last line forever: a character
    /// who goes silent once you have run out of things to say reads as a bug, not
    /// as an ending.
    pub fn next_line(&self, id: NpcId) -> &'static str {
        let lines = npc(id).lines;
        lines[self.count(id).min(lines.len() - 1)]
    }
}

/// How long a line stays on screen before it clears itself.
///
/// Long enough to read at `UiScale` 1, short enough to be gone before you walk
/// into the next room.
pub const SPEECH_SECONDS: f32 = 6.0;

#[derive(Resource)]
pub(crate) struct SpeechTimer(Timer);

impl Default for SpeechTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(SPEECH_SECONDS, TimerMode::Once))
    }
}

/// Above the room art and item sprites, but *below* nothing else: the click
/// target has to sit above the hotspot layer, because the way out of the concierge
/// is a hotspot and she stands in front of it. Picking takes the top entity at a
/// point, so a click aimed at her has to land on her and not on the door behind.
const NPC_SPRITE_Z: f32 = 0.6;
const NPC_TARGET_Z: f32 = 1.5;

/// Invisible, but it has to be a sprite for the picking backend to see it.
const TARGET_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);

// ------------------------------------------------------------------ spawning

/// Puts every character belonging to the current room on screen, and takes away
/// the ones that are not here.
///
/// Both the sprite and its target are reconciled, because a character removed from
/// the table would otherwise leave its click target behind: still invisible, still
/// eating clicks, now pointing at nobody.
pub(crate) fn spawn_npc_sprites(
    rooms: Query<(Entity, &RoomDef), With<Room>>,
    sprites: NpcChildren<NpcSprite>,
    targets: NpcChildren<NpcTarget>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let Ok((room, def)) = rooms.single() else {
        return;
    };
    let here = def.variants[0].path;

    let mut present: Vec<NpcId> = sprites.iter().map(|(_, sprite)| sprite.0).collect();
    for def in npcs() {
        if def.room != here || present.contains(&def.id) {
            continue;
        }
        let id = def.id;
        commands.entity(room).with_children(|parent| {
            parent.spawn((
                RoomPart,
                NpcSprite(id),
                Sprite {
                    image: asset_server.load(def.texture),
                    custom_size: Some(def.size),
                    ..default()
                },
                Transform::from_xyz(def.pos.x, def.pos.y, NPC_SPRITE_Z),
                // The sprite is not clickable; the target below it is. Picking
                // both would make one figure need two clicks.
                Pickable::IGNORE,
            ));
            parent.spawn((
                RoomPart,
                NpcTarget(id),
                Sprite::from_color(TARGET_COLOR, def.hit),
                Transform::from_xyz(def.pos.x, def.pos.y, NPC_TARGET_Z),
                Pickable::default(),
            ));
        });
        present.push(id);
    }

    for (entity, sprite) in &sprites {
        if npc(sprite.0).room != here {
            commands.entity(entity).despawn();
        }
    }
    for (entity, target) in &targets {
        if npc(target.0).room != here {
            commands.entity(entity).despawn();
        }
    }
}

// ----------------------------------------------------------------- speaking

/// What a click means, given whether somebody is already talking.
///
/// The interesting rule of this interaction is that a click means two different
/// things depending on whether the dialogue box is up: with nothing on screen it
/// starts a conversation, and with a line on screen it takes the line away no
/// matter where you clicked.
///
/// Split out as a pure function because that rule is the whole feature and it is
/// easy to break by moving one line between the two arms.
pub enum Clicked {
    /// Nobody is talking and this click was on a character: start a conversation.
    Speak(NpcId),
    /// Something is on screen: put it away, wherever the click landed.
    Dismiss,
    /// Nothing on screen and the click missed every character.
    Nothing,
}

/// Whether two rectangles share any area. Touching edges do not count.
///
/// The same test the hotspot editor uses for its overlap readout, reached across
/// rather than duplicated: the editor is compiled out without its feature, and this
/// has to hold in a plain build.
fn rects_overlap(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2) -> bool {
    let (ahw, ahh) = (a_size.x / 2.0, a_size.y / 2.0);
    let (bhw, bhh) = (b_size.x / 2.0, b_size.y / 2.0);
    (a_pos.x - ahw < b_pos.x + bhw)
        && (b_pos.x - bhw < a_pos.x + ahw)
        && (a_pos.y - ahh < b_pos.y + bhh)
        && (b_pos.y - bhh < a_pos.y + ahh)
}

/// Decides what a click means.
///
/// `on_npc` is the character that was clicked, if any. A click anywhere else is
/// still a dismiss while the box is up, which is what makes "click anywhere" work
/// without a full-screen button: there is nothing to catch it, because this system
/// sees every click in the game rather than only the ones that hit something.
pub fn classify_click(speaking: Option<NpcId>, on_npc: Option<NpcId>) -> Clicked {
    match (speaking, on_npc) {
        (Some(_), _) => Clicked::Dismiss,
        (None, Some(id)) => Clicked::Speak(id),
        (None, None) => Clicked::Nothing,
    }
}

/// Turns clicks into a line on screen, or into no line at all.
///
/// Reads *every* click rather than only the ones that landed on a character, which
/// is how the box gets dismissed by clicking the wall. It does not need a
/// full-screen button for that: `Pointer<Click>` carries every click in the app,
/// including the ones that hit nothing pickable.
///
/// The click is not consumed - a `MessageReader` cannot consume - so clicking a
/// door while she is talking both puts the line away and walks the player through.
/// That is the wanted behaviour rather than a leak: the alternative is that a click
/// aimed at the door does nothing at all, because something invisible is in the
/// way, and a player who cannot leave the room until a timer expires cannot leave
/// the room.
///
/// Gated on `gameplay_active` for the same reason the hotspot system is: with the
/// editor open a click means "select this", and dialogue over the selection readout
/// would be worse than useless.
pub(crate) fn npc_click_system(
    mut clicks: MessageReader<Pointer<Click>>,
    targets: Query<&NpcTarget>,
    rooms: Query<&RoomDef, With<Room>>,
    mut talked: ResMut<TalkedTo>,
    mut speech: ResMut<Speech>,
    mut timer: ResMut<SpeechTimer>,
) {
    let Ok(room) = rooms.single() else {
        return;
    };
    for click in clicks.read() {
        let on_npc = targets.get(click.entity).ok().map(|target| target.0);
        match classify_click(speech.who, on_npc) {
            Clicked::Speak(id) => {
                // Read the line *before* counting the conversation, so the first
                // one gets the first line rather than the second.
                let line = talked.next_line(id);
                speech.say(id, line, room.variants[0].path);
                talked.once_more(id);
                timer.0.reset();
            }
            Clicked::Dismiss => speech.clear(),
            Clicked::Nothing => {}
        }
    }
}

/// Clears the line once its time is up, so the screen does not keep showing the
/// last thing anyone said for the rest of the game.
pub(crate) fn speech_timeout_system(
    time: Res<Time>,
    mut timer: ResMut<SpeechTimer>,
    mut speech: ResMut<Speech>,
) {
    if speech.who.is_none() {
        return;
    }
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        speech.clear();
    }
}

/// Clears the line on a room change: what she says belongs to the room the player
/// is standing in.
///
/// The room is remembered when the line is spoken rather than watched for a room
/// change, because "the player walked away mid-sentence" and "the line timed out"
/// want the same result and the timeout already exists.
pub(crate) fn clear_speech_on_move(rooms: Query<&RoomDef, With<Room>>, mut speech: ResMut<Speech>) {
    if speech.who.is_none() {
        return;
    }
    let Ok(def) = rooms.single() else {
        return;
    };
    if speech.room != Some(def.variants[0].path) {
        speech.clear();
    }
}

// -------------------------------------------------------------------- plugin

/// Every NPC picture, for the loading overlay in `rooms::data::all_paths`.
pub(crate) fn paths() -> impl Iterator<Item = &'static str> {
    data::paths()
}

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TalkedTo>()
            .init_resource::<Speech>()
            .init_resource::<SpeechTimer>()
            .add_systems(OnEnter(GameState::Game), spawn_speech_ui)
            .add_systems(
                Update,
                (
                    spawn_npc_sprites,
                    npc_click_system.run_if(crate::scenes::game::gameplay_active),
                    clear_speech_on_move,
                    speech_timeout_system,
                    update_speech_visibility,
                    update_speech_text,
                )
                    .chain()
                    .run_if(in_state(GameState::Game)),
            );
    }
}
