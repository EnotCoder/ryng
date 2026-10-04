use bevy::audio::AudioSource;
use bevy::prelude::*;

use crate::scenes::game::rooms::components::Room;
use crate::scenes::game::rooms::data::RoomDef;
use crate::state::GameState;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TransitionSound {
    /// The default: an ordinary door, which is most of them. A door that makes a
    /// point has its own sound, so falling back to silence would be wrong - it would
    /// read as a missing asset rather than as an ordinary room.
    #[default]
    NextRoom,
    NextRoomWithOpenDoor,
    ElevatorFall,
    None,
}

fn path_for(sound: &TransitionSound) -> Option<&'static str> {
    match sound {
        TransitionSound::NextRoom => Some("sounds/next_room.mp3"),
        TransitionSound::NextRoomWithOpenDoor => Some("sounds/next_room_with_open_door.mp3"),
        TransitionSound::ElevatorFall => Some("sounds/elevator_fall.mp3"),
        TransitionSound::None => None,
    }
}

#[derive(Component)]
pub struct PlayingTransitionSound;

pub fn play_transition_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    sound: &TransitionSound,
) {
    if let Some(path) = path_for(sound) {
        let handle: Handle<AudioSource> = asset_server.load(path);
        commands.spawn((
            AudioPlayer(handle),
            PlaybackSettings::DESPAWN,
            PlayingTransitionSound,
        ));
    }
}

/// A one-off effect that is not a room transition: a card hitting a counter, a
/// toy being set down. Despawns itself when it finishes.
#[derive(Component)]
pub struct PlayingItemSound;

pub fn play_item_sound(commands: &mut Commands, asset_server: &AssetServer) {
    let handle: Handle<AudioSource> = asset_server.load("sounds/card_fall.mp3");
    commands.spawn((
        AudioPlayer(handle),
        PlaybackSettings::DESPAWN,
        PlayingItemSound,
    ));
}

#[derive(Component)]
pub struct PlayingBackgroundMusic(pub &'static str);

/// Which ambience loop a room plays. This used to be derived by string-matching
/// the room's asset path, with the city rooms listed by hand; a new street room
/// silently got the wrong track. The table now says which one it wants.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Music {
    /// No ambience.
    #[default]
    Indoors,
    City,
    Basement,
}

fn music_path(music: Music) -> &'static str {
    match music {
        Music::City => "sounds/city.mp3",
        Music::Basement => "sounds/basement.mp3",
        Music::Indoors => "sounds/null_room.mp3",
    }
}

fn spawn_background_music(commands: &mut Commands, asset_server: &AssetServer, path: &'static str) {
    let handle: Handle<AudioSource> = asset_server.load(path);
    commands.spawn((
        PlayingBackgroundMusic(path),
        AudioPlayer(handle),
        PlaybackSettings::LOOP,
        DespawnOnExit(GameState::Game),
    ));
}

pub(crate) fn background_music_system(
    rooms: Query<&RoomDef, With<Room>>,
    music: Query<(Entity, &PlayingBackgroundMusic)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let Ok(def) = rooms.single() else {
        return;
    };
    let desired = music_path(def.music);

    if let Some((entity, current)) = music.iter().next() {
        if current.0 != desired {
            commands.entity(entity).despawn();
            spawn_background_music(&mut commands, &asset_server, desired);
        }
        return;
    }
    spawn_background_music(&mut commands, &asset_server, desired);
}
