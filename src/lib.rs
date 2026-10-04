use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::prelude::*;

pub mod acts;
pub mod buttons;
pub mod scenes;
pub mod state;

use state::GameState;

pub const DEBUG_SHOW_HOTSPOTS: bool = false;

/// Whether the hotspot editor is running.
///
/// This is the feature, not a separate switch: `cargo run --features
/// hotspot-editor` turns the editor on and a plain `cargo run` compiles it out
/// entirely. There is deliberately no `const` here, because a constant that
/// silently disables a whole module is worse than not having one at all - it looks
/// like the editor is running and it is not.
pub const DEBUG_HOTSPOT_EDIT: bool = cfg!(feature = "hotspot-editor");

pub const DESIGN_HEIGHT: f32 = 720.0;

/// Half-width and half-height of the visible frame, in the same units as room
/// art. The camera is `FixedVertical` at `DESIGN_HEIGHT`, so this is fixed.
pub const FRAME_HALF: Vec2 = Vec2::new(640.0, 360.0);

/// How far the UI scale is allowed to move away from 1.0.
///
/// The lower bound keeps the HUD from collapsing on a very short window, where
/// `s.px` would round most constants to nothing. The upper bound stops a tall
/// window from inflating the UI past the size it was authored at - past that it
/// looks oversized rather than proportional.
const UI_SCALE_MIN: f32 = 0.4;
const UI_SCALE_MAX: f32 = 2.5;

/// Design-space to screen scale. UI code multiplies its constants by this so a
/// layout authored against `DESIGN_HEIGHT` keeps its proportions.
#[derive(Resource, Clone, Copy)]
pub struct UiScale(pub f32);

impl Default for UiScale {
    fn default() -> Self {
        Self(1.0)
    }
}

impl UiScale {
    /// A design-space length as a `Val::Px`, already scaled.
    pub fn px(&self, design: f32) -> Val {
        Val::Px(design * self.0)
    }

    /// A design-space font size, already scaled.
    pub fn font(&self, design: f32) -> FontSize {
        FontSize::Px(design * self.0)
    }
}

#[bevy_main]
pub fn main() {
    // Bound on its own because the builder chain borrows the app: keeping it in a
    // `let` is what lets the chain be one statement and the inspector be added
    // after it, rather than both being fused into one expression.
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins((
            scenes::loading::LoadingPlugin,
            scenes::intro::IntroPlugin,
            scenes::menu::MenuPlugin,
            scenes::settings::SettingsPlugin,
            scenes::game::GamePlugin,
        ))
        .init_state::<GameState>()
        .init_resource::<UiScale>()
        .add_systems(Startup, spawn_camera)
        .add_systems(PreUpdate, update_ui_scale)
        .add_systems(Update, scenes::loading::loading_system);

    // The ECS inspector, drawn as a floating window over the game: the entity
    // hierarchy, the components on the selected entity, and the resource and asset
    // lists. A separate statement rather than another entry in the tuple above, so
    // the list a plain build compiles is exactly the list it was before the
    // inspector was added.
    //
    // `EguiPlugin` is bevy_egui's own bridge - the inspector draws through it -
    // and `WorldInspectorPlugin` is the crate's drop-in window over the world.
    // Both are needed: the first opens the context, the second fills it. Nothing
    // has to be attached to the camera by hand: bevy_egui gives the first camera
    // that appears a primary context of its own.
    //
    // `cfg` and not a `const`: without the feature the crate is not even linked,
    // so a switch that could be quietly set to false would be a lie - the
    // inspector would look present in the source and be absent from the build.
    // This is the same reasoning as `DEBUG_HOTSPOT_EDIT` above.
    #[cfg(feature = "inspector")]
    app.add_plugins((
        bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
        bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
    ));

    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn(Camera2d)
        .insert(Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: DESIGN_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }));
}

fn update_ui_scale(window: Single<&Window>, mut ui_scale: ResMut<UiScale>) {
    let logical_height = window.height() / window.scale_factor();
    ui_scale.0 = (logical_height / DESIGN_HEIGHT).clamp(UI_SCALE_MIN, UI_SCALE_MAX);
}
