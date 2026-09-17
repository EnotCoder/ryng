use bevy::prelude::*;

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
) {
    let pending = fade.pending;
    let mut finished = false;
    match &mut fade.phase {
        FadePhase::Idle => {}
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
                    spawn_room(
                        &mut commands,
                        &asset_server,
                        path,
                        &def.hotspots,
                        Vec3::new(0.0, 0.0, 0.0),
                        def.title,
                    );
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