use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::prelude::*;

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
        .init_state::<GameState>()
        .init_resource::<UiScale>()
        .init_resource::<scenes::fade::RoomFade>()
        .add_systems(Startup, spawn_camera)
        .add_systems(PreUpdate, update_ui_scale)
        .add_systems(OnEnter(GameState::Menu), scenes::menu::spawn_menu_ui)
        .add_systems(OnEnter(GameState::Game), scenes::game::spawn_game_ui)
        .add_systems(Update, (
            scenes::menu::menu_button_system,
            scenes::game::game_button_system,
            scenes::game::game_hotspot_system,
            scenes::fade::room_fade_system,
            scenes::game::update_room_label,
        ))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d).insert(Projection::Orthographic(
        OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: DESIGN_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        },
    ));
}

fn update_ui_scale(window: Single<&Window>, mut ui_scale: ResMut<UiScale>) {
    let logical_height = window.height() / window.scale_factor();
    ui_scale.0 = (logical_height / DESIGN_HEIGHT).clamp(0.4, 2.5);
}