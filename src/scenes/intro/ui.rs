use bevy::audio::AudioSource;
use bevy::prelude::*;

use crate::scenes::intro::systems::{LOGO_SCALE, LOGO_START_Y, Logo};
use crate::state::GameState;

pub fn spawn_intro_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Sprite::from_image(asset_server.load("tex/main_fon.png")),
        Transform::from_scale(Vec3::new(3.0, 3.0, 1.0)),
        DespawnOnExit(GameState::Intro),
    ));

    commands.spawn((
        Logo,
        Sprite::from_image(asset_server.load("tex/game_logo.png")),
        Transform {
            translation: Vec3::new(0.0, LOGO_START_Y, 2.0),
            scale: Vec3::splat(LOGO_SCALE),
            ..default()
        },
        DespawnOnExit(GameState::Intro),
    ));

    commands.spawn((
        AudioPlayer::<AudioSource>(asset_server.load("sounds/intro.mp3")),
        PlaybackSettings::DESPAWN,
        DespawnOnExit(GameState::Intro),
    ));
}
