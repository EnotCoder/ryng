use bevy::prelude::*;

use crate::state::GameState;

mod systems;
mod ui;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<systems::SoundVolume>()
            .init_resource::<systems::VignetteSettings>()
            .init_resource::<systems::SettingsPanelOpen>()
            .add_systems(OnEnter(GameState::Menu), ui::spawn_settings_panel)
            .add_systems(
                Update,
                (
                    systems::settings_button_system,
                    systems::close_button_system,
                    systems::slider_input_system,
                    systems::checkbox_click_system,
                    systems::panel_visibility_system,
                    systems::slider_update_system,
                    systems::checkbox_update_system,
                    systems::apply_settings_system,
                ),
            );
    }
}
