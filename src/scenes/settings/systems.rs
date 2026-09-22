use bevy::audio::{GlobalVolume, Volume};
use bevy::post_process::effect_stack::Vignette;
use bevy::prelude::*;

use crate::scenes::menu::MenuAction;

pub const VIGNETTE_ON_INTENSITY: f32 = 0.9;

pub(super) const ACCENT_ON: Color = Color::srgb(0.95, 0.85, 0.35);
pub(super) const ACCENT_OFF: Color = Color::srgb(0.25, 0.25, 0.3);

#[derive(Resource)]
pub struct SoundVolume(pub f32);

impl Default for SoundVolume {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Resource)]
pub struct VignetteSettings {
    pub enabled: bool,
}

impl Default for VignetteSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Resource, Default)]
pub struct SettingsPanelOpen(pub bool);

/// Volume slider track. `width` and `thumb` are in logical pixels (already scaled by `UiScale`).
#[derive(Component)]
pub struct Slider {
    pub width: f32,
    pub thumb: f32,
}

#[derive(Component)]
pub struct SliderFill;

#[derive(Component)]
pub struct SliderThumb;

#[derive(Component)]
pub struct VolumeLabel;

#[derive(Component)]
pub struct VignetteCheckbox;

#[derive(Component)]
pub struct CheckMark;

#[derive(Component)]
pub enum SettingsPanelAction {
    Close,
}

#[derive(Component)]
pub struct SettingsPanel;

pub fn settings_button_system(
    mut interactions: Query<(&Interaction, &MenuAction), (Changed<Interaction>, With<Button>)>,
    mut open: ResMut<SettingsPanelOpen>,
) {
    for (interaction, action) in &mut interactions {
        if *interaction == Interaction::Pressed && matches!(action, MenuAction::Settings) {
            open.0 = !open.0;
        }
    }
}

pub fn close_button_system(
    mut interactions: Query<
        (&Interaction, &SettingsPanelAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut open: ResMut<SettingsPanelOpen>,
) {
    for (interaction, action) in &mut interactions {
        if *interaction == Interaction::Pressed && matches!(action, SettingsPanelAction::Close) {
            open.0 = false;
        }
    }
}

pub fn slider_input_system(
    mut presses: MessageReader<Pointer<Press>>,
    mut moves: MessageReader<Pointer<Move>>,
    mut releases: MessageReader<Pointer<Release>>,
    sliders: Query<Entity, With<Slider>>,
    mut volume: ResMut<SoundVolume>,
    mut active: Local<Option<Entity>>,
) {
    for press in presses.read() {
        if sliders.contains(press.entity) {
            *active = Some(press.entity);
            if let Some(fraction) = fraction_from(press.event.hit.position) {
                volume.0 = fraction;
            }
        }
    }
    for movement in moves.read() {
        if let Some(active_id) = *active
            && movement.entity == active_id
            && let Some(fraction) = fraction_from(movement.event.hit.position)
        {
            volume.0 = fraction;
        }
    }
    for _release in releases.read() {
        *active = None;
    }
}

fn fraction_from(position: Option<Vec3>) -> Option<f32> {
    position.map(|pos| (pos.x + 0.5).clamp(0.0, 1.0))
}

pub fn checkbox_click_system(
    mut interactions: Query<
        (&Interaction, &VignetteCheckbox),
        (Changed<Interaction>, With<Button>),
    >,
    mut vignette: ResMut<VignetteSettings>,
) {
    for (interaction, _checkbox) in &mut interactions {
        if *interaction == Interaction::Pressed {
            vignette.enabled = !vignette.enabled;
        }
    }
}

pub fn panel_visibility_system(
    open: Res<SettingsPanelOpen>,
    mut panels: Query<&mut Visibility, With<SettingsPanel>>,
) {
    let visibility = if open.0 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut panel in &mut panels {
        if *panel != visibility {
            *panel = visibility;
        }
    }
}

pub fn slider_update_system(
    volume: Res<SoundVolume>,
    sliders: Query<&Slider>,
    mut fills: Query<&mut Node, (With<SliderFill>, Without<SliderThumb>)>,
    mut thumbs: Query<&mut Node, (With<SliderThumb>, Without<SliderFill>)>,
    mut labels: Query<&mut Text, With<VolumeLabel>>,
) {
    if !volume.is_changed() {
        return;
    }
    let fraction = volume.0.clamp(0.0, 1.0);
    for mut fill in &mut fills {
        fill.width = Val::Percent(fraction * 100.0);
    }
    if let Some(slider) = sliders.iter().next() {
        for mut thumb in &mut thumbs {
            thumb.left = Val::Px(fraction * slider.width - slider.thumb / 2.0);
        }
    }
    for mut label in &mut labels {
        label.0 = format!("{}%", (fraction * 100.0).round() as u32);
    }
}

pub fn checkbox_update_system(
    vignette: Res<VignetteSettings>,
    mut boxes: Query<&mut BackgroundColor, With<VignetteCheckbox>>,
    mut checks: Query<&mut Visibility, With<CheckMark>>,
) {
    if !vignette.is_changed() {
        return;
    }
    for mut checkbox in &mut boxes {
        checkbox.0 = if vignette.enabled {
            ACCENT_ON
        } else {
            ACCENT_OFF
        };
    }
    for mut check in &mut checks {
        *check = if vignette.enabled {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn apply_settings_system(
    volume: Res<SoundVolume>,
    vignette: Res<VignetteSettings>,
    mut global_volume: ResMut<GlobalVolume>,
    mut camera_vignettes: Query<&mut Vignette>,
) {
    if volume.is_changed() {
        global_volume.volume = Volume::Linear(volume.0.clamp(0.0, 1.0));
    }
    if vignette.is_changed() {
        let intensity = if vignette.enabled {
            VIGNETTE_ON_INTENSITY
        } else {
            0.0
        };
        for mut camera_vignette in &mut camera_vignettes {
            camera_vignette.intensity = intensity;
        }
    }
}
