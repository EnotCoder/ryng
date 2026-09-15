use bevy::prelude::*;
use std::collections::HashSet;

use crate::state::GameState;

pub const NORMAL_BUTTON: Color = Color::srgb(0.2, 0.6, 0.9);
pub const HOVERED_BUTTON: Color = Color::srgb(0.4, 0.8, 0.6);
pub const PRESSED_BUTTON: Color = Color::srgb(0.9, 0.3, 0.3);

pub const BUTTON_SIZE: Vec2 = Vec2::new(150.0, 50.0);
pub const BUTTON_GAP: f32 = 50.0;
pub const FONT_SIZE: FontSize = FontSize::Px(20.0);

#[derive(Component)]
pub enum MenuAction {
    Settings,
    Play,
    Quit,
}

#[derive(Component)]
pub enum GameAction {
    Back
}

pub fn draw_button(
    parent: &mut ChildSpawnerCommands<'_>, text: &str, action: impl Component,
){
    parent.spawn((
        Button,
        action,
        Node {
            width: Val::Px(BUTTON_SIZE.x),
            height: Val::Px(BUTTON_SIZE.y),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Interaction::default(),
        BackgroundColor(NORMAL_BUTTON),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new(text),
            TextFont {
                font_size: FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

pub fn button_system(
    mut query: Query<
        (Entity, &Interaction, &MenuAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut was_pressed: Local<HashSet<Entity>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (entity, interaction, action, mut color) in &mut query {
        if click_visual(interaction, &mut color, &mut was_pressed, entity) {
            fire_menu(action, &mut next_state, &mut exit);
        }
    }
}

pub fn game_button_system(
    mut query: Query<
        (Entity, &Interaction, &GameAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut was_pressed: Local<HashSet<Entity>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, interaction, action, mut color) in &mut query {
        if click_visual(interaction, &mut color, &mut was_pressed, entity) {
            fire_game(action, &mut next_state);
        }
    }
}

fn click_visual(
    interaction: &Interaction,
    color: &mut BackgroundColor,
    was_pressed: &mut HashSet<Entity>,
    entity: Entity,
) -> bool {
    match *interaction {
        Interaction::Pressed => {
            was_pressed.insert(entity);
            color.0 = PRESSED_BUTTON;
            false
        }
        Interaction::Hovered => {
            let released = was_pressed.remove(&entity);
            color.0 = HOVERED_BUTTON;
            released
        }
        Interaction::None => {
            let released = was_pressed.remove(&entity);
            color.0 = NORMAL_BUTTON;
            released
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

fn fire_game(action: &GameAction, next: &mut NextState<GameState>) {
    match action {
        GameAction::Back => next.set(GameState::Menu),
    }
}