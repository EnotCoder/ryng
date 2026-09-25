use bevy::audio::AudioSource;
use bevy::prelude::*;

use crate::scenes::game::rooms::components::{Room, RoomVariantIndex, RoomVariants};
use crate::state::GameState;

pub enum TransitionSound {
    NextRoom,
    NextRoomWithOpenDoor,
    ElevatorFall,
    None,
}

impl Default for TransitionSound {
    fn default() -> Self {
        Self::NextRoom
    }
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

#[derive(Component)]
pub struct PlayingBackgroundMusic(pub &'static str);

const CITY_ROOMS: [&str; 2] = [
    "tex/rooms/floor_1/street_to_home_1.png",
    "tex/rooms/floor_1/street_to_home_2.png",
];

fn background_music_path(room_path: &str) -> &'static str {
    if CITY_ROOMS.contains(&room_path) {
        "sounds/city.mp3"
    } else if room_path.starts_with("tex/rooms/basement/") {
        "sounds/basement.mp3"
    } else {
        "sounds/null_room.mp3"
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

pub fn background_music_system(
    rooms: Query<(&RoomVariants, &RoomVariantIndex), With<Room>>,
    music: Query<(Entity, &PlayingBackgroundMusic)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let Ok((variants, index)) = rooms.single() else {
        return;
    };
    let desired = background_music_path(variants.0[index.0].path);

    if let Some((entity, current)) = music.iter().next() {
        if current.0 != desired {
            commands.entity(entity).despawn();
            spawn_background_music(&mut commands, &asset_server, desired);
        }
        return;
    }
    spawn_background_music(&mut commands, &asset_server, desired);
}
