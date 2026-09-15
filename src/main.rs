use bevy::prelude::*;

mod button;
mod state;

use crate::button::*;
use crate::state::*;

fn main() {
    //Create app
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(GameState::Menu), spawn_menu_ui)
        .add_systems(OnEnter(GameState::Game), spawn_game_ui)
        .add_systems(Update, (button_system, game_button_system))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_game_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        DespawnOnExit(GameState::Game), // автоудаление при выходе из Game
    ))
    .with_children(|parent| {
        draw_button(parent, "Back", GameAction::Back);
    });

    commands.spawn((
        Sprite::from_image(asset_server.load("room.png")),
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0), // позиция (Godot position)
            scale: Vec3::new(1.2, 1.2, 1.0),      // Godot scale
            ..default()
        },
        DespawnOnExit(GameState::Game),
    ));
}

fn spawn_menu_ui(mut commands: Commands){
    //Ui
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(BUTTON_GAP),
            ..default()
        },
        DespawnOnExit(GameState::Menu),
    ))
    .with_children(|parent| {
        draw_button(parent, "Settings", MenuAction::Settings);
        draw_button(parent, "Play", MenuAction::Play);
        draw_button(parent, "Quit", MenuAction::Quit);
    });
}