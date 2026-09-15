use bevy::prelude::*;
use std::collections::HashSet;

use crate::state::GameState;

pub const NORMAL_BUTTON: Color = Color::srgb(0.2, 0.6, 0.9);
pub const HOVERED_BUTTON: Color = Color::srgb(0.4, 0.8, 0.6);
pub const PRESSED_BUTTON: Color = Color::srgb(0.9, 0.3, 0.3);

const NORMAL_TINT: Color = Color::WHITE;
const HOVERED_TINT: Color = Color::srgb(0.8, 0.8, 0.8);
const PRESSED_TINT: Color = Color::srgb(0.5, 0.5, 0.5); 

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

pub fn draw_button_with_texture(
    parent: &mut ChildSpawnerCommands<'_>, text: &str, action: impl Component,
    asset_server: &Res<AssetServer>
){
    parent.spawn((
        Button,
        action,
        ImageNode {
            image: asset_server.load("button_tex.png"),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
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
        (Entity, &Interaction, &MenuAction, Option<&mut BackgroundColor>, Option<&mut ImageNode>),
        (Changed<Interaction>, With<Button>),
    >,
    mut was_pressed: Local<HashSet<Entity>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (entity, interaction, action, mut bg, mut img) in &mut query {
        let visual = click_visual(interaction, &mut was_pressed, entity);
        if let Some(c) = bg.as_mut() { c.0 = visual.bg_color; }
        if let Some(i) = img.as_mut() { i.color = visual.tint; }
        if visual.released { fire_menu(action, &mut next_state, &mut exit); }
    }
}

pub fn game_button_system(
    mut query: Query<
        (Entity, &Interaction, &GameAction, Option<&mut BackgroundColor>, Option<&mut ImageNode>),
        (Changed<Interaction>, With<Button>),
    >,
    mut was_pressed: Local<HashSet<Entity>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, interaction, action, mut bg, mut img) in &mut query {
        let visual = click_visual(interaction, &mut was_pressed, entity);
        if let Some(c) = bg.as_mut() { c.0 = visual.bg_color; }
        if let Some(i) = img.as_mut() { i.color = visual.tint; }
        if visual.released { fire_game(action, &mut next_state); }
    }
}

struct VisualState {
    bg_color: Color,
    tint: Color,
    released: bool,
}

fn click_visual(
    interaction: &Interaction,
    was_pressed: &mut HashSet<Entity>,
    entity: Entity,
) -> VisualState {
    match *interaction {
        Interaction::Pressed => {
            was_pressed.insert(entity);
            VisualState { bg_color: PRESSED_BUTTON, tint: PRESSED_TINT, released: false }
        }
        Interaction::Hovered => {
            let released = was_pressed.remove(&entity);
            VisualState { bg_color: HOVERED_BUTTON, tint: HOVERED_TINT, released }
        }
        Interaction::None => {
            let released = was_pressed.remove(&entity);
            VisualState { bg_color: NORMAL_BUTTON, tint: NORMAL_TINT, released }
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