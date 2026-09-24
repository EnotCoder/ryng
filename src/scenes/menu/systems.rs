use bevy::prelude::*;
use std::collections::HashSet;

use crate::UiScale;
use crate::buttons;
use crate::scenes::menu::ui::MenuAction;
use crate::scenes::settings::SettingsPanelOpen;
use crate::state::GameState;

pub fn menu_button_system(
    mut query: Query<
        (
            Entity,
            &Interaction,
            &MenuAction,
            Option<&mut BackgroundColor>,
            Option<&mut ImageNode>,
            Option<&mut Node>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut was_pressed: Local<HashSet<Entity>>,
    settings_open: Res<SettingsPanelOpen>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
    ui_scale: Res<UiScale>,
) {
    for (entity, interaction, action, bg, img, node) in &mut query {
        let visual = buttons::click_visual(interaction, &mut was_pressed, entity, ui_scale.0);
        if buttons::apply_visual(visual, bg, img, node)
            && !settings_open.0
        {
            fire_menu(action, &mut next_state, &mut exit);
        }
    }
}

fn fire_menu(
    action: &MenuAction,
    next: &mut NextState<GameState>,
    exit: &mut MessageWriter<AppExit>,
) {
    match action {
        MenuAction::Play => next.set(GameState::Game),
        MenuAction::Quit => {
            exit.write(AppExit::Success);
        }
        MenuAction::Settings => {}
    }
}
