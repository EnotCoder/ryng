use bevy::prelude::*;

use crate::UiScale;
use crate::buttons;
use crate::scenes::loading::spawn_loading_overlay;
use crate::state::GameState;

// Menu backdrop: the logo sits above the buttons, the full-screen picture behind
// both. Both are scaled rather than sized, so they stay proportional to their art.
const BACKDROP_SCALE: f32 = 3.0;
const LOGO_SCALE: f32 = 1.5;
const LOGO_OFFSET_Y: f32 = 200.0;

#[derive(Component)]
pub enum MenuAction {
    Settings,
    Play,
    Quit,
}

pub fn spawn_menu_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_scale: Res<UiScale>,
) {
    let s = *ui_scale;
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: s.px(buttons::BUTTON_GAP),
                ..default()
            },
            DespawnOnExit(GameState::Menu),
        ))
        .with_children(|parent| {
            buttons::draw_button_with_texture(
                parent,
                "Settings",
                MenuAction::Settings,
                &asset_server,
                s.0,
            );
            buttons::draw_button_with_texture(parent, "Play", MenuAction::Play, &asset_server, s.0);
            buttons::draw_button_with_texture(parent, "Quit", MenuAction::Quit, &asset_server, s.0);
        });

    commands.spawn((
        Sprite::from_image(asset_server.load("tex/game_logo.png")),
        Transform {
            translation: Vec3::new(0.0, LOGO_OFFSET_Y, 1.0),
            scale: Vec3::new(LOGO_SCALE, LOGO_SCALE, LOGO_SCALE),
            ..default()
        },
        DespawnOnExit(GameState::Menu),
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("tex/main_fon.png")),
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(BACKDROP_SCALE, BACKDROP_SCALE, 1.0),
            ..default()
        },
        DespawnOnExit(GameState::Menu),
    ));

    let fon = asset_server.load("tex/main_fon.png");
    spawn_loading_overlay(&mut commands, vec![fon], s.0, GameState::Menu);
}
