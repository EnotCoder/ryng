use bevy::prelude::*;

mod button;
use crate::button::*;

fn main() {
    //Create app
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, spawn_menu_ui)
        .add_systems(Update, button_system)
        .run();
}

fn spawn_menu_ui(mut commands: Commands){
    commands.spawn(Camera2d);

    //Ui
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        column_gap: Val::Px(BUTTON_GAP),
        ..default()
    })
    .with_children(|parent| {
        draw_button(parent, "Settings", on_click_settings);
        draw_button(parent, "Play", on_click_play);
        draw_button(parent, "Quit", on_click_quit);
    });
}