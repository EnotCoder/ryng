use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::prelude::*;

pub mod acts;
pub mod buttons;
pub mod scenes;
pub mod state;

use state::GameState;

pub const DEBUG_SHOW_HOTSPOTS: bool = false;
pub const DESIGN_HEIGHT: f32 = 720.0;

#[derive(Resource)]
pub struct UiScale(pub f32);

impl Default for UiScale {
    fn default() -> Self {
        Self(1.0)
    }
}

#[bevy_main]
pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((scenes::menu::MenuPlugin, scenes::game::GamePlugin))
        .init_state::<GameState>()
        .init_resource::<UiScale>()
        .add_systems(Startup, spawn_camera)
        .add_systems(PreUpdate, update_ui_scale)
        .add_systems(Update, scenes::loading::loading_system)
        .run();
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
    ui_scale.0 = (logical_height / DESIGN_HEIGHT).clamp(0.4, 2.5);
}
