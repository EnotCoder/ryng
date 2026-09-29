use bevy::prelude::*;

use crate::state::GameState;

mod systems;
mod ui;

pub use systems::SettingsPanelOpen;

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
                    // Generic over the resource they edit, so a new setting is a
                    // type plus one line here rather than a pair of systems.
                    systems::slider_input_system::<systems::SoundVolume>,
                    systems::slider_update_system::<systems::SoundVolume>,
                    systems::checkbox_click_system::<systems::VignetteSettings>,
                    systems::checkbox_update_system::<systems::VignetteSettings>,
                    systems::panel_visibility_system,
                    systems::apply_settings_system,
                )
                    // The panel is spawned on `OnEnter(Menu)` and despawns on exit,
                    // so none of these have anything to touch in the other states.
                    // `SoundVolume` and `VignetteSettings` are resources and outlive
                    // the state, so a setting chosen here stays applied in game.
                    .run_if(in_state(GameState::Menu)),
            );
    }
}
