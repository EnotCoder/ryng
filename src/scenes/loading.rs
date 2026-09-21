use bevy::prelude::*;

use crate::state::GameState;

#[derive(Component)]
pub struct LoadingOverlay {
    pending: Vec<Handle<Image>>,
    fade: Option<Timer>,
}

pub fn spawn_loading_overlay(
    commands: &mut Commands,
    pending: Vec<Handle<Image>>,
    ui_scale: f32,
    exit_state: GameState,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            Pickable::IGNORE,
            LoadingOverlay {
                pending,
                fade: None,
            },
            DespawnOnExit(exit_state),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Loading..."),
                TextFont {
                    font_size: FontSize::Px(30.0 * ui_scale),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn loading_system(
    time: Res<Time>,
    assets: Res<Assets<Image>>,
    mut overlays: Query<(Entity, &mut LoadingOverlay, &mut BackgroundColor)>,
    mut commands: Commands,
) {
    for (entity, mut overlay, mut bg) in &mut overlays {
        if overlay.fade.is_none() {
            if overlay.pending.iter().all(|h| assets.get(h).is_some()) {
                overlay.fade = Some(Timer::from_seconds(0.25, TimerMode::Once));
            }
        }
        if let Some(fade) = &mut overlay.fade {
            fade.tick(time.delta());
            bg.0.set_alpha(1.0 - fade.fraction());
            if fade.is_finished() {
                commands.entity(entity).despawn();
            }
        }
    }
}
