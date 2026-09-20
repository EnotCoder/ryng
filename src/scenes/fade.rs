use bevy::prelude::*;
use crate::acts::{CurrentAct};
use crate::scenes::game::room_def;
use crate::scenes::hotspot::{spawn_room, Room};
use crate::state::GameState;

pub const FADE_DURATION: f32 = 0.35;

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
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        Pickable::IGNORE,
        FadeOverlay,
        DespawnOnExit(GameState::Game),
    ));
}

pub fn room_fade_system(
    time: Res<Time>,
    mut fade: ResMut<RoomFade>,
    mut overlays: Query<&mut BackgroundColor, With<FadeOverlay>>,
    rooms: Query<Entity, With<Room>>,
    active_sounds: Query<Entity, With<crate::scenes::sound::PlayingTransitionSound>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut current_act: ResMut<CurrentAct>,
) {
    let pending = fade.pending;
    let mut finished = false;
    match &mut fade.phase {
        FadePhase::Idle => {
            let auto = fade.auto_timer.take();
            if let Some((path, mut timer)) = auto {
                timer.tick(time.delta());
                if timer.is_finished() {
                    fade.pending = Some(path);
                    fade.phase =
                        FadePhase::FadeOut(Timer::from_seconds(FADE_DURATION, TimerMode::Once));
                } else {
                    fade.auto_timer = Some((path, timer));
                }
            }
        }
        FadePhase::FadeOut(timer) => {
            timer.tick(time.delta());
            set_overlay_alpha(&mut overlays, timer.fraction());
            finished = timer.is_finished();
            if finished {
                if let Some(path) = pending {
                    if let Ok(old) = rooms.single() {
                        commands.entity(old).despawn();
                    }
                    let def = room_def(path);

                    if let Some(next_act_id) = def.next_act {
                        current_act.0 = next_act_id;
                    }

                    spawn_room(
                        &mut commands,
                        &asset_server,
                        def.variants.clone(),
                        Vec3::ZERO,
                        def.interactive,
                    );
                    fade.auto_timer = def.auto_next.map(|(next, secs)| {
                        (next, Timer::from_seconds(secs, TimerMode::Once))
                    });
                    for sound in &active_sounds {
                        commands.entity(sound).despawn();
                    }
                    crate::scenes::sound::play_transition_sound(
                        &mut commands,
                        &asset_server,
                        &def.sound,
                    );
                }
            }
        }
        FadePhase::FadeIn(timer) => {
            timer.tick(time.delta());
            set_overlay_alpha(&mut overlays, 1.0 - timer.fraction());
            finished = timer.is_finished();
        }
    }
    if finished {
        match fade.phase {
            FadePhase::FadeOut(_) => {
                fade.phase = FadePhase::FadeIn(Timer::from_seconds(FADE_DURATION, TimerMode::Once));
            }
            FadePhase::FadeIn(_) => {
                fade.phase = FadePhase::Idle;
                fade.pending = None;
            }
            FadePhase::Idle => {}
        }
    }
}

fn set_overlay_alpha(overlays: &mut Query<&mut BackgroundColor, With<FadeOverlay>>, alpha: f32) {
    for mut bg in overlays {
        bg.0.set_alpha(alpha);
    }
}