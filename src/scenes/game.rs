use bevy::prelude::*;
use std::collections::HashSet;

use crate::buttons;
use crate::state::GameState;

#[derive(Component)]
pub enum GameAction {
    Back,
}

pub fn spawn_game_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::End,
            align_items: AlignItems::End,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        DespawnOnExit(GameState::Game),
    ))
    .with_children(|parent| {
        buttons::draw_button(parent, "Back", GameAction::Back);
    });

    commands.spawn((
        Sprite::from_image(asset_server.load("room.png")),
        Transform {
            translation: Vec3::new(-250.0, 0.0, 0.0),
            scale: Vec3::new(1.2, 1.2, 1.0),
            ..default()
        },
        DespawnOnExit(GameState::Game),
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("main_fon.png")),
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(3.0, 3.0, 1.0),
            ..default()
        },
        DespawnOnExit(GameState::Game),
    ));
}

pub fn game_button_system(
    mut query: Query<
        (Entity, &Interaction, &GameAction, Option<&mut BackgroundColor>, Option<&mut ImageNode>),
        (Changed<Interaction>, With<Button>),
    >,
    mut was_pressed: Local<HashSet<Entity>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, interaction, action, bg, img) in &mut query {
        let visual = buttons::click_visual(interaction, &mut was_pressed, entity);
        if buttons::apply_visual(visual, bg, img) {
            fire_game(action, &mut next_state);
        }
    }
}

fn fire_game(action: &GameAction, next: &mut NextState<GameState>) {
    match action {
        GameAction::Back => next.set(GameState::Menu),
    }
}