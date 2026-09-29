use bevy::prelude::*;
use std::collections::HashSet;

use crate::UiScale;
use crate::buttons;
use crate::scenes::menu::ui::MenuAction;
use crate::scenes::settings::SettingsPanelOpen;
use crate::state::GameState;

pub fn menu_button_system(
    clicks: buttons::ButtonQuery<MenuAction>,
    mut was_pressed: Local<HashSet<Entity>>,
    settings_open: Res<SettingsPanelOpen>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
    ui_scale: Res<UiScale>,
) {
    buttons::for_each_click(clicks, &mut was_pressed, ui_scale, |action| {
        // Visuals are applied even while the panel is open; only the action is
        // suppressed, so the underlying buttons keep their hover state.
        if !settings_open.0 {
            fire_menu(action, &mut next_state, &mut exit);
        }
    });
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
