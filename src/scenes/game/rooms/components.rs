use bevy::prelude::*;

use crate::acts::Item;

#[derive(Component, Clone, Copy, PartialEq)]
pub enum HotspotAction {
    GoToRoom(&'static str),
    /// Pick the item up into the first free inventory slot.
    Take(Item),
    /// Put the item down, if it is held.
    Drop(Item),
}

#[derive(Component)]
pub struct Hotspot;

#[derive(Component)]
pub struct HotspotIcon;

/// One of the four bars that draw a hotspot's outline.
///
/// A component rather than a tag on the hotspot itself, because the bars are
/// children: they ride along with the room's breathing motion and are despawned
/// with the room, and they are positioned relative to the hotspot rather than in
/// world coordinates.
#[derive(Component)]
pub struct HotspotOutline;

/// How thick the hover outline is drawn, in design pixels.
///
/// 6 rather than the 5 this started at: at 1:1 that is a 5px hairline, and the
/// room art is photographic, so a thin bright line on it is easy to lose against
/// the clutter - the first version was measurably invisible over a light door. The
/// bars overlap the hotspot's own rectangle by this much on every side, so the
/// outline traces the edge of the clickable area rather than sitting just inside
/// it.
pub const HOTSPOT_OUTLINE_THICKNESS: f32 = 6.0;

/// The white of the hover outline. A warm white would blend into the yellow lamps
/// in the rooms and into the white signage; plain white reads against everything
/// in the game.
pub const OUTLINE_COLOR: Color = Color::WHITE;

/// Above the hotspot it belongs to, which is picked at [`HOTSPOT_Z`], and below
/// an NPC's click target, which is the whole reason the ordering is a constant
/// rather than an inline `0.1`: the concierge's way out is a hotspot and the
/// granny stands in front of it, so an outline drawn over her target would eat
/// the clicks meant for her. See [`crate::scenes::game::npc`].
///
/// The bars are `Pickable::IGNORE` anyway, so they cannot take a click; this is
/// about what they cover, not what they receive.
pub const HOTSPOT_OUTLINE_Z: f32 = 1.25;

/// The size the blinking arrow is drawn at, whatever the hotspot underneath it
/// happens to be. It used to be derived from the hotspot rectangle, which made
/// the arrow grow with every door that was resized and gave it a different size
/// on the item spots than on the doors.
pub const HOTSPOT_ICON_SIZE: Vec2 = Vec2::splat(10.0);

#[derive(Component)]
pub struct Room;

#[derive(Component)]
pub struct RoomTitle(pub &'static str);

#[derive(Component)]
pub struct RoomStory(pub &'static str);

/// Children (sprite + hotspots) that are replaced on variant switch.
#[derive(Component)]
pub struct RoomPart;

#[derive(Component)]
pub struct RoomVariants(pub &'static [RoomVariant]);

#[derive(Component)]
pub struct RoomVariantIndex(pub usize);

#[derive(Clone, Copy)]
pub struct RoomVariant {
    pub path: &'static str,
    pub title: &'static str,
    pub story: &'static str,
    pub hotspots: &'static [HotspotDef],
    /// The carousel control that brings the player *to this shot*, or `None` for a
    /// shot that is not somewhere the carousel goes.
    ///
    /// Belonging to the destination rather than to the shot being left is what makes
    /// the table readable: the button shows the picture of where a press will take
    /// you, so the art has to be found by looking up the shot it leads to, not the
    /// one on screen.
    ///
    /// `None` on a single-shot room, which has nowhere to go and so no control.
    pub preview: Option<&'static str>,
}

/// How long one frame of a shot-to-shot flip stays on screen.
///
/// A per-frame time rather than a total, because the art is a pan of fixed length:
/// 12 frames at 20fps is 0.6s. Slower than [`crate::scenes::fade::FADE_DURATION`]
/// on purpose - the frames are motion-blurred, and cutting through them any faster
/// turns the pan into a flicker.
pub const FLIP_FRAME_SECONDS: f32 = 1.0 / 20.0;

/// How long a room's animation takes, in seconds.
///
/// The whole animation's length rather than a per-frame time, and then divided by
/// the number of frames when the component is built. A per-frame constant cannot
/// be right for both of the things that decide how many frames there are - the
/// art and the room the player is standing in. Nineteen frames at a rate chosen
/// for fifty-eight play in under two seconds and leave the rest of the beat on a
/// still, and the same rate over a shorter list runs past the room's own
/// `auto_next` and gets cut off mid-fall. Deriving one from the other means a
/// list of any length fills its room exactly.
///
/// Deliberately not [`FLIP_FRAME_SECONDS`], which is a per-frame time because a
/// pan is a fixed piece of art with a fixed length. The lift's animation is
/// matched to the sound that goes with it instead, and the sound is matched to
/// the room, so this number belongs to the room rather than to the picture.
pub const ANIM_SECONDS: f32 = 5.0;

/// A room that plays its own pictures while it stands, without the player asking.
///
/// The lift's fall is the reason this exists: it is a beat, so there is no
/// carousel to press and nothing to click, and it cannot be a pan either - a pan
/// runs between two shots of a room, and this room has one shot and a film
/// instead. So the room carries its frames and a system runs them.
///
/// Present for as long as the room is on screen. Unlike [`RoomFlip`] nothing
/// waits on it: the room moves on by its own `auto_next`, so the last frame is
/// simply the one still up when the transition takes the room away.
#[derive(Component)]
pub struct RoomAnim {
    /// The frames, in the order the art was drawn.
    pub frames: &'static [&'static str],
    /// How many frames have been shown so far, the first one included.
    shown: usize,
    /// Time left on the frame currently on screen.
    pub timer: Timer,
}

impl RoomAnim {
    /// Start playing `frames`, showing the first one straight away.
    ///
    /// The list is spread over [`ANIM_SECONDS`] however long it is. An empty list
    /// would divide by zero, so it counts as one frame: a room carrying an empty
    /// animation has nothing to play either way, and the guard is what keeps an
    /// infinity out of the timer.
    pub fn new(frames: &'static [&'static str]) -> Self {
        let per_frame = ANIM_SECONDS / frames.len().max(1) as f32;
        Self {
            frames,
            shown: 0,
            timer: Timer::from_seconds(per_frame, TimerMode::Once),
        }
    }

    /// How long one frame lasts, for the list this was built with.
    ///
    /// Exposed so a test can check the timing without reaching past the timer.
    /// That the answer is a function of the frame count is the whole point, and
    /// it is otherwise only visible from outside as a number that happens to be
    /// right today.
    pub fn frame_seconds(&self) -> f32 {
        ANIM_SECONDS / self.frames.len().max(1) as f32
    }

    /// The frame that belongs on screen now.
    pub fn frame(&self) -> &'static str {
        self.frames[self.shown.min(self.frames.len() - 1)]
    }

    /// Show the next frame, or report that every frame has had its turn.
    ///
    /// The same shape as [`RoomFlip::advance`], and for the same reason: each
    /// frame gets its full slot before the next replaces it, so the last one is
    /// not stepped over by the tick that would have replaced it.
    pub fn advance(&mut self) -> bool {
        self.shown += 1;
        self.timer.reset();
        self.shown < self.frames.len()
    }
}

/// The pan from one shot of a room to the next, playing on the room while it runs.
///
/// Present only for as long as the frames are on screen: the carousel system reads
/// its absence to decide the player may change shot again, and the flip system
/// removes it the moment the destination shot is spawned. The outgoing shot's
/// hotspots are already despawned, so a room mid-pan is a picture with nothing to
/// click - which is what stops a click landing on a door the player is not looking
/// at yet.
#[derive(Component)]
pub struct RoomFlip {
    /// The frames, in the order the art was drawn: from the first shot towards the
    /// last.
    pub frames: &'static [&'static str],
    /// The shot the player is heading for.
    pub to: usize,
    /// How many frames have been shown so far, the first one included.
    shown: usize,
    /// Which way the player went. The art runs from the first shot to the last, so
    /// this alone decides whether the list plays forwards or backwards.
    pub forward: bool,
    /// Time left on the frame currently on screen.
    pub timer: Timer,
}

impl RoomFlip {
    /// Start a flip towards `to`, showing the first frame straight away.
    pub fn new(frames: &'static [&'static str], to: usize, forward: bool) -> Self {
        Self {
            frames,
            to,
            shown: 0,
            forward,
            timer: Timer::from_seconds(FLIP_FRAME_SECONDS, TimerMode::Once),
        }
    }

    /// The frame that belongs on screen now.
    ///
    /// Counted from whichever end the player set off from, so the two directions are
    /// one list read from opposite ends and the art only has to exist once.
    pub fn frame(&self) -> &'static str {
        let index = if self.forward {
            self.shown
        } else {
            self.frames.len() - 1 - self.shown
        };
        self.frames[index]
    }

    /// Show the next frame, or report that every frame has had its turn.
    ///
    /// Each frame gets its full slot before the next replaces it, the last one
    /// included: landing on the same tick that showed it would skip the twelfth
    /// frame, and the pan would stop one step short of where the art ends.
    pub fn advance(&mut self) -> bool {
        self.shown += 1;
        self.timer.reset();
        self.shown < self.frames.len()
    }
}

#[derive(Component, Clone, Copy, PartialEq)]
pub struct HotspotDef {
    pub action: HotspotAction,
    pub pos: Vec2,
    pub size: Vec2,
    /// Spend this one item to pass, as the concierge desk does with the pass.
    /// Empty for an ordinary door.
    pub gate: Option<Item>,
    /// Pass only while carrying all of these. Unlike `gate` they are not
    /// consumed - the tools on the floor 2 door stay in the bag afterwards.
    pub require: &'static [Item],
}
