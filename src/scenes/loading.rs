use bevy::prelude::*;

use crate::scenes::game::StartRoom;
use crate::state::GameState;

#[derive(Component)]
pub struct LoadingOverlay {
    /// Visible only through the real spawner; a test that needs an overlay up can
    /// ask for this rather than reaching into the fields.
    pub(crate) pending: Vec<Handle<Image>>,
    pub(crate) fade: Option<Timer>,
}

impl LoadingOverlay {
    /// An overlay that is up and holding nothing - already clear, but present.
    ///
    /// For the systems that treat "an overlay exists" as "the room is not visible
    /// yet", which is a different question from "are the pictures here". A test
    /// needs the first without the second, and the fields are not public because
    /// nothing in the game should be building one by hand.
    pub fn blocking() -> Self {
        Self {
            pending: Vec::new(),
            fade: None,
        }
    }
}

/// A handle to every picture the game will ever show, for as long as it is running.
///
/// The overlay waits for these and then despawns, and that despawn used to take the
/// only handle to each picture with it. Bevy drops an asset once nothing holds a
/// strong handle to it, so every picture that was not on screen at that moment went
/// back to being unloaded - which is why the room the player is standing in works
/// and the one two doors away does not until it is walked into.
///
/// Rooms got away with it because a room change happens behind a fade, and a fade
/// covers a picture that is still arriving. A carousel pan does not: the player
/// presses an arrow, the shot is swapped for a frame that is not there yet, and
/// there is no fade over it - just the backdrop, because the pan despawns the shot
/// it was going to replace. Holding the handles here is what makes the pan show the
/// pan.
///
/// Not a general-purpose cache and deliberately not: this is exactly the preload
/// list, kept for exactly as long as the session is.
#[derive(Resource, Default)]
pub struct PreloadedImages(Vec<Handle<Image>>);

impl PreloadedImages {
    /// Take the pictures the game needs and keep them.
    pub fn hold(handles: Vec<Handle<Image>>) -> Self {
        Self(handles)
    }

    /// The handles being held.
    ///
    /// Exposed so a test can check what is still held once the overlay that used to
    /// hold it is gone, which is the whole point of the resource.
    pub fn handles(&self) -> &[Handle<Image>] {
        &self.0
    }
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
            .init_resource::<PreloadedImages>()
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
