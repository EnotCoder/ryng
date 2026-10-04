use bevy::prelude::*;

use crate::scenes::game::StartRoom;
use crate::state::GameState;

#[derive(Component)]
pub struct LoadingOverlay {
    pending: Vec<Handle<Image>>,
    fade: Option<Timer>,
}

pub const SPLASH_SECONDS: f32 = 1.0;

/// How long the overlay takes to clear once the last texture is in.
///
/// Short on purpose: by then the player has already seen the splash, and a long
/// fade here reads as the game being slow rather than as a transition.
const FADE_OUT_SECONDS: f32 = 0.25;

/// The preload overlay's label. Sized in design space and scaled like the rest of
/// the UI, so it stays the same size relative to the screen as everything else.
const OVERLAY_TEXT_SIZE: f32 = 30.0;

#[derive(Resource)]
pub struct SplashTimer(Timer);

impl Default for SplashTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(SPLASH_SECONDS, TimerMode::Once))
    }
}

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SplashTimer>()
            .add_systems(OnEnter(GameState::Loading), spawn_splash_ui)
            .add_systems(Update, splash_system.run_if(in_state(GameState::Loading)));
    }
}

pub fn spawn_splash_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::BLACK),
        Pickable::IGNORE,
        DespawnOnExit(GameState::Loading),
    ));
}

pub fn splash_system(
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
    mut next: ResMut<NextState<GameState>>,
    start_room: Res<StartRoom>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        // `--rooms` wants the room and nothing else, so the intro and the menu are
        // exactly what it is skipping: straight to the game, with `spawn_game_ui`
        // opening the room that was asked for.
        //
        // The splash stays either way. It is one second and it is what covers the
        // first frame of asset loading, so dropping it would only add a flash.
        next.set(if start_room.0.is_some() {
            GameState::Game
        } else {
            GameState::Intro
        });
    }
}

pub fn spawn_loading_overlay(
    commands: &mut Commands,
    pending: Vec<Handle<Image>>,
    ui_scale: f32,
    exit_state: GameState,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            Pickable::IGNORE,
            LoadingOverlay {
                pending,
                fade: None,
            },
            DespawnOnExit(exit_state),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Loading..."),
                TextFont {
                    font_size: FontSize::Px(OVERLAY_TEXT_SIZE * ui_scale),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn loading_system(
    time: Res<Time>,
    assets: Res<Assets<Image>>,
    mut overlays: Query<(Entity, &mut LoadingOverlay, &mut BackgroundColor)>,
    mut commands: Commands,
) {
    for (entity, mut overlay, mut bg) in &mut overlays {
        // Only arm the fade once everything has arrived, and never re-arm it: the
        // `is_none` check is what stops a still-loading overlay from restarting the
        // fade every frame.
        if overlay.fade.is_none() && overlay.pending.iter().all(|h| assets.get(h).is_some()) {
            overlay.fade = Some(Timer::from_seconds(FADE_OUT_SECONDS, TimerMode::Once));
        }
        if let Some(fade) = &mut overlay.fade {
            fade.tick(time.delta());
            bg.0.set_alpha(1.0 - fade.fraction());
            if fade.is_finished() {
                commands.entity(entity).despawn();
            }
        }
    }
}
