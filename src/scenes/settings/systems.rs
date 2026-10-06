use bevy::audio::{GlobalVolume, Volume};
use bevy::ecs::component::Mutable;
use bevy::prelude::*;

use crate::UiScale;

use crate::scenes::menu::MenuAction;

#[derive(Resource)]
pub struct SoundVolume(pub f32);

impl Default for SoundVolume {
    fn default() -> Self {
        Self(1.0)
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
///
/// The design-space originals are kept alongside so the track can be rebuilt when
/// the window changes: `width` and `thumb` are read by the click handling to turn a
/// press into a position, so a stale value makes the slider answer to the wrong
/// part of the window.
#[derive(Component)]
pub struct Slider<R: SliderValue> {
    pub width: f32,
    pub thumb: f32,
    /// Design-space track width, replayed on resize by `rescale_sliders`.
    pub design_width: f32,
    /// Design-space thumb size, replayed on resize by `rescale_sliders`.
    pub design_thumb: f32,
    pub marker: std::marker::PhantomData<fn() -> R>,
}

#[derive(Component)]
pub struct SliderFill;

#[derive(Component)]
pub struct SliderThumb;

/// The percentage label beside the track.
#[derive(Component)]
pub struct SliderReadout;

// ------------------------------------------------------------------ panel

/// Menu buttons, for the system below that watches them.
///
/// Named because the generic version is unreadable at the point it matters: the
/// `With<Button>` is what stops this firing on every widget in the menu, and
/// `Changed` is what makes it edge-triggered rather than per-frame.
type MenuButtonQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static MenuAction),
    (Changed<Interaction>, With<Button>),
>;

pub fn settings_button_system(
    mut interactions: MenuButtonQuery,
    mut open: ResMut<SettingsPanelOpen>,
) {
    for (interaction, action) in &mut interactions {
        if *interaction == Interaction::Pressed && matches!(action, MenuAction::Settings) {
            open.0 = !open.0;
        }
    }
}

/// Buttons inside the settings panel itself, carrying the panel's own action.
type PanelButtonQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static SettingsPanelAction),
    (Changed<Interaction>, With<Button>),
>;

pub fn close_button_system(
    mut interactions: PanelButtonQuery,
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

/// Reapplies the design-space track width and thumb size after a resize.
///
/// A separate system from `rescale_ui_system` because `Slider` is generic over its
/// value type, and a system cannot be added to the app without naming one
/// concretely. The settings screen has a single value type; a second slider of
/// another type means another line in `add_systems` here.
pub fn rescale_slider_system<R: SliderValue + 'static>(
    scale: Res<UiScale>,
    mut sliders: Query<&mut Slider<R>>,
) {
    if !scale.is_changed() {
        return;
    }
    for mut slider in &mut sliders {
        slider.width = slider.design_width * scale.0;
        slider.thumb = slider.design_thumb * scale.0;
    }
}

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

// ------------------------------------------------------------------ apply

pub fn apply_settings_system(volume: Res<SoundVolume>, mut global_volume: ResMut<GlobalVolume>) {
    if volume.is_changed() {
        global_volume.volume = Volume::Linear(volume.0.clamp(0.0, 1.0));
    }
}
