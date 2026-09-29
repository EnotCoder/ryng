use bevy::audio::{GlobalVolume, Volume};
use bevy::ecs::component::Mutable;
use bevy::post_process::effect_stack::Vignette;
use bevy::prelude::*;

use crate::scenes::menu::MenuAction;
use crate::scenes::settings::ui::{ACCENT_OFF, ACCENT_ON};

pub const VIGNETTE_ON_INTENSITY: f32 = 0.9;

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

#[derive(Component)]
pub enum SettingsPanelAction {
    Close,
}

#[derive(Component)]
pub struct SettingsPanel;

// --------------------------------------------------------------- widgets
//
// A setting is described by the resource it edits, so a new one costs a type, an
// `impl` block and one line in `mod.rs`, rather than a pair of systems.

/// Bound every generic system here needs. `Resource` is a supertrait of
/// `Component` and says nothing about mutability, so the `Component` bound has
/// to be spelled out for `ResMut` to be constructible.
pub(crate) trait SettingResource: Resource + Component<Mutability = Mutable> {}
impl<T: Resource + Component<Mutability = Mutable>> SettingResource for T {}

/// A value edited by dragging, rendered as `0..=1` with a readout.
pub trait SliderValue {
    fn fraction(&self) -> f32;
    fn set_fraction(&mut self, value: f32);
    /// The text next to the track, e.g. `42%`.
    fn readout(&self) -> String;
}

impl SliderValue for SoundVolume {
    fn fraction(&self) -> f32 {
        self.0.clamp(0.0, 1.0)
    }
    fn set_fraction(&mut self, value: f32) {
        self.0 = value.clamp(0.0, 1.0);
    }
    fn readout(&self) -> String {
        format!("{}%", (self.fraction() * 100.0).round() as u32)
    }
}

/// The slider track for `R`. `width` and `thumb` are logical pixels, already
/// scaled.
#[derive(Component)]
pub struct Slider<R: SliderValue> {
    pub width: f32,
    pub thumb: f32,
    pub marker: std::marker::PhantomData<fn() -> R>,
}

#[derive(Component)]
pub struct SliderFill;

#[derive(Component)]
pub struct SliderThumb;

/// The percentage label beside the track.
#[derive(Component)]
pub struct SliderReadout;

/// A bool value flipped by clicking.
pub trait ToggleValue {
    fn enabled(&self) -> bool;
    fn toggle(&mut self);
}

impl ToggleValue for VignetteSettings {
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

#[derive(Component)]
pub struct Checkbox<T: ToggleValue> {
    pub marker: std::marker::PhantomData<fn() -> T>,
}

// ------------------------------------------------------------------ panel

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

// ----------------------------------------------------------------- slider

/// Press, drag, release. Generic over the resource so a new slider needs no new
/// system.
pub fn slider_input_system<R: SettingResource + SliderValue>(
    mut presses: MessageReader<Pointer<Press>>,
    mut moves: MessageReader<Pointer<Move>>,
    mut releases: MessageReader<Pointer<Release>>,
    sliders: Query<Entity, With<Slider<R>>>,
    mut value: ResMut<R>,
    mut active: Local<Option<Entity>>,
) {
    for press in presses.read() {
        if sliders.contains(press.entity) {
            *active = Some(press.entity);
            if let Some(fraction) = fraction_from(press.event.hit.position) {
                value.set_fraction(fraction);
            }
        }
    }
    for movement in moves.read() {
        if let Some(active_id) = *active
            && movement.entity == active_id
            && let Some(fraction) = fraction_from(movement.event.hit.position)
        {
            value.set_fraction(fraction);
        }
    }
    for _release in releases.read() {
        *active = None;
    }
}

/// A pointer hit arrives in normalized space, so `x` runs -0.5..0.5.
pub(super) fn fraction_from(position: Option<Vec3>) -> Option<f32> {
    position.map(|pos| (pos.x + 0.5).clamp(0.0, 1.0))
}

/// Paints track, thumb and readout from the resource.
pub fn slider_update_system<R: SettingResource + SliderValue>(
    value: Res<R>,
    sliders: Query<&Slider<R>>,
    mut fills: Query<&mut Node, (With<SliderFill>, Without<SliderThumb>)>,
    mut thumbs: Query<&mut Node, (With<SliderThumb>, Without<SliderFill>)>,
    mut readouts: Query<&mut Text, With<SliderReadout>>,
) {
    if !value.is_changed() {
        return;
    }
    let fraction = value.fraction();
    for mut fill in &mut fills {
        fill.width = Val::Percent(fraction * 100.0);
    }
    if let Some(slider) = sliders.iter().next() {
        for mut thumb in &mut thumbs {
            thumb.left = Val::Px(fraction * slider.width - slider.thumb / 2.0);
        }
    }
    let text = value.readout();
    for mut readout in &mut readouts {
        readout.0 = text.clone();
    }
}

// --------------------------------------------------------------- checkbox

pub fn checkbox_click_system<T: SettingResource + ToggleValue>(
    mut interactions: Query<(&Interaction, &Checkbox<T>), (Changed<Interaction>, With<Button>)>,
    mut value: ResMut<T>,
) {
    for (interaction, _checkbox) in &mut interactions {
        if *interaction == Interaction::Pressed {
            value.toggle();
        }
    }
}

pub fn checkbox_update_system<T: SettingResource + ToggleValue>(
    value: Res<T>,
    mut boxes: Query<&mut BackgroundColor, With<Checkbox<T>>>,
) {
    if !value.is_changed() {
        return;
    }
    let color = if value.enabled() {
        ACCENT_ON
    } else {
        ACCENT_OFF
    };
    for mut checkbox in &mut boxes {
        checkbox.0 = color;
    }
}

// ------------------------------------------------------------------ apply

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
