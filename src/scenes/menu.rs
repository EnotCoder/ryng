use bevy::prelude::*;
use std::collections::HashSet;

use crate::buttons;
use crate::scenes::loading::spawn_loading_overlay;
use crate::state::GameState;
use crate::UiScale;

#[derive(Component)]
pub enum MenuAction {
    Settings,
    Play,
    Quit,
}

pub fn spawn_menu_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_scale: Res<UiScale>,
) {
    let s = ui_scale.0;
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(buttons::BUTTON_GAP * s),
            ..default()
        },
        DespawnOnExit(GameState::Menu),
    ))
    .with_children(|parent| {
        buttons::draw_button_with_texture(parent, "Settings", MenuAction::Settings, &asset_server, s);
        buttons::draw_button_with_texture(parent, "Play", MenuAction::Play, &asset_server, s);
        buttons::draw_button_with_texture(parent, "Quit", MenuAction::Quit, &asset_server, s);
    });

    commands.spawn((
        Sprite::from_image(asset_server.load("tex/game_logo.png")),
        Transform {
            translation: Vec3::new(0.0, 200.0, 1.0),
            scale: Vec3::new(1.5, 1.5, 1.5),
            ..default()
        },
        DespawnOnExit(GameState::Menu),
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("tex/main_fon.png")),
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(3.0, 3.0, 1.0),
            ..default()
        },
        DespawnOnExit(GameState::Menu),
    ));

    let fon = asset_server.load("tex/main_fon.png");
    spawn_loading_overlay(&mut commands, vec![fon], s, GameState::Menu);
}

pub fn menu_button_system(
    mut query: Query<
        (Entity, &Interaction, &MenuAction, Option<&mut BackgroundColor>, Option<&mut ImageNode>, Option<&mut Node>),
        (Changed<Interaction>, With<Button>),
    >,
    mut was_pressed: Local<HashSet<Entity>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
    ui_scale: Res<UiScale>,
) {
    for (entity, interaction, action, bg, img, node) in &mut query {
        let visual = buttons::click_visual(interaction, &mut was_pressed, entity, ui_scale.0);
        if buttons::apply_visual(visual, bg, img, node) {
            fire_menu(action, &mut next_state, &mut exit);
        }
    }
}

fn fire_menu(action: &MenuAction, next: &mut NextState<GameState>, exit: &mut MessageWriter<AppExit>) {
    match action {
        MenuAction::Play => next.set(GameState::Game),
        MenuAction::Quit => { exit.write(AppExit::Success); },
        MenuAction::Settings => println!("Settings"),
    }
}