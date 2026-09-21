use bevy::audio::AudioSource;
use bevy::prelude::*;

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
