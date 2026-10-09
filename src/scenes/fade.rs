use crate::acts::CurrentAct;
use crate::scenes::game::rooms::data::room_def;
use crate::scenes::game::rooms::{components::Room, spawn::spawn_room};
use crate::scenes::loading::LoadingOverlay;
use crate::scenes::sound::{PlayingTransitionSound, play_transition_sound};
use crate::state::GameState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

pub const FADE_DURATION: f32 = 0.35;

/// The overlay starts fully transparent, because it is spawned before the first
/// fade and a solid black flash at the start of a room would be visible.
const OVERLAY_CLEAR: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);

#[derive(Default)]
pub enum FadePhase {
    #[default]
    Idle,
    FadeOut(Timer),
    FadeIn(Timer),
}

#[derive(Resource, Default)]
pub struct RoomFade {
    pub phase: FadePhase,
    pub pending: Option<&'static str>,
    pub auto_timer: Option<(&'static str, Timer)>,
}

#[derive(Component)]
pub struct FadeOverlay;

pub fn spawn_fade_overlay(commands: &mut Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(OVERLAY_CLEAR),
        Pickable::IGNORE,
        FadeOverlay,
        DespawnOnExit(GameState::Game),
    ));
}

// The transition used to be one system doing four jobs. Each phase is now its
// own system, and the three are chained in `game::mod`, so reading any one of
// them tells you the whole of what happens during that phase.

/// Ticks the timer that hands a non-interactive room over to the next one.
///
/// Held while the preload overlay is up, for the same reason
/// [`crate::scenes::game::systems::room_anim_system`] holds its frames: a room is
/// spawned before its pictures have arrived, and a beat that counts down under
/// the overlay is over before the player has seen any of it. The timer is put
/// back rather than dropped, which is what holds the beat at full length instead
/// of shortening it by however long the load took.
pub fn auto_next_system(
    time: Res<Time>,
    mut fade: ResMut<RoomFade>,
    loading: Query<Entity, With<LoadingOverlay>>,
) {
    let FadePhase::Idle = fade.phase else {
        return;
    };
    let Some((path, mut timer)) = fade.auto_timer.take() else {
        return;
    };
    if !loading.is_empty() {
        fade.auto_timer = Some((path, timer));
        return;
    }
    timer.tick(time.delta());
    if timer.is_finished() {
        fade.pending = Some(path);
        fade.phase = FadePhase::FadeOut(fade_timer());
    } else {
        fade.auto_timer = Some((path, timer));
    }
}

/// The entities a transition has to clean up before the next room appears.
///
/// Both are read-only and both are used only to despawn, so they travel together:
/// grouped they read as "what a transition clears", and ungrouped they read as two
/// unrelated queries.
#[derive(SystemParam)]
pub struct StaleEntities<'w, 's> {
    rooms: Query<'w, 's, Entity, With<Room>>,
    sounds: Query<'w, 's, Entity, With<PlayingTransitionSound>>,
}

/// Darkens the screen, then swaps the room while it is fully black.
pub fn fade_out_system(
    time: Res<Time>,
    mut fade: ResMut<RoomFade>,
    mut overlays: Query<&mut BackgroundColor, With<FadeOverlay>>,
    stale: StaleEntities,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut current_act: ResMut<CurrentAct>,
) {
    let FadePhase::FadeOut(timer) = &mut fade.phase else {
        return;
    };
    timer.tick(time.delta());
    set_overlay_alpha(&mut overlays, timer.fraction());
    if !timer.is_finished() {
        return;
    }

    let Some(path) = fade.pending else {
        fade.phase = FadePhase::FadeIn(fade_timer());
        return;
    };
    if let Ok(old) = stale.rooms.single() {
        commands.entity(old).despawn();
    }
    let def = room_def(path, current_act.0);

    if let Some(next_act_id) = def.next_act {
        current_act.0 = next_act_id;
    }

    spawn_room(&mut commands, &asset_server, def, Vec3::ZERO);
    fade.auto_timer = def
        .auto_next
        .map(|(next, secs)| (next, Timer::from_seconds(secs, TimerMode::Once)));
    for sound in &stale.sounds {
        commands.entity(sound).despawn();
    }
    play_transition_sound(&mut commands, &asset_server, &def.sound);

    fade.phase = FadePhase::FadeIn(fade_timer());
}

/// Brightens the screen back up and re-enables input.
pub fn fade_in_system(
    time: Res<Time>,
    mut fade: ResMut<RoomFade>,
    mut overlays: Query<&mut BackgroundColor, With<FadeOverlay>>,
) {
    let FadePhase::FadeIn(timer) = &mut fade.phase else {
        return;
    };
    timer.tick(time.delta());
    set_overlay_alpha(&mut overlays, 1.0 - timer.fraction());
    if timer.is_finished() {
        fade.phase = FadePhase::Idle;
        fade.pending = None;
    }
}

fn fade_timer() -> Timer {
    Timer::from_seconds(FADE_DURATION, TimerMode::Once)
}

fn set_overlay_alpha(overlays: &mut Query<&mut BackgroundColor, With<FadeOverlay>>, alpha: f32) {
    for mut bg in overlays {
        bg.0.set_alpha(alpha);
    }
}
