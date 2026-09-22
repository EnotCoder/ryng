use bevy::prelude::*;

use crate::state::GameState;

pub const LOGO_SCALE: f32 = 3.0;
const LOGO_HALF: f32 = 150.0 * LOGO_SCALE / 2.0;
pub const LOGO_START_Y: f32 = crate::DESIGN_HEIGHT / 2.0 + LOGO_HALF + 50.0;
pub const LOGO_LAND_Y: f32 = 0.0;
pub const LOGO_EXIT_Y: f32 = -(crate::DESIGN_HEIGHT / 2.0 + LOGO_HALF + 50.0);
pub const FALL_START: f32 = 0.2;
pub const FALL_END: f32 = 1.0;
pub const HOLD_END: f32 = 2.0;
pub const INTRO_TOTAL: f32 = 4.776;

#[derive(Component)]
pub struct Logo;

#[derive(Resource)]
pub struct IntroTimer(Timer);

impl Default for IntroTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(INTRO_TOTAL, TimerMode::Once))
    }
}

fn intro_position(t: f32) -> (f32, f32) {
    if t < FALL_START {
        (LOGO_START_Y, 0.0)
    } else if t < FALL_END {
        let x = ((t - FALL_START) / (FALL_END - FALL_START)).clamp(0.0, 1.0);
        let fall = x * x;
        (LOGO_START_Y + (LOGO_LAND_Y - LOGO_START_Y) * fall, fall)
    } else if t < HOLD_END {
        (LOGO_LAND_Y, 1.0)
    } else {
        let x = ((t - HOLD_END) / (INTRO_TOTAL - HOLD_END)).clamp(0.0, 1.0);
        (LOGO_LAND_Y + (LOGO_EXIT_Y - LOGO_LAND_Y) * x, 1.0)
    }
}

pub fn intro_system(
    time: Res<Time>,
    mut timer: ResMut<IntroTimer>,
    mut logos: Query<(&mut Transform, &mut Sprite), With<Logo>>,
    mut next: ResMut<NextState<GameState>>,
) {
    timer.0.tick(time.delta());
    let t = timer.0.elapsed_secs();
    if timer.0.is_finished() {
        next.set(GameState::Menu);
    }
    let (y, alpha) = intro_position(t);
    for (mut transform, mut sprite) in &mut logos {
        transform.translation.y = y;
        sprite.color.set_alpha(alpha);
    }
}
