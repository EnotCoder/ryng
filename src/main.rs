use bevy::prelude::*;

mod buttons;
mod scenes;
mod state;

use state::GameState;

pub const DEBUG_SHOW_HOTSPOTS: bool = false;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .init_resource::<scenes::fade::RoomFade>()
        .add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(GameState::Menu), scenes::menu::spawn_menu_ui)
        .add_systems(OnEnter(GameState::Game), scenes::game::spawn_game_ui)
        .add_systems(Update, (
            scenes::menu::menu_button_system,
            scenes::game::game_button_system,
            scenes::game::game_hotspot_system,
            scenes::fade::room_fade_system,
            scenes::game::update_room_label,
        ))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}