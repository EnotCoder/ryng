//! Tests for the intro logo animation.
//!
//! `intro_position` is the whole animation: four chained segments driven by
//! wall-clock time, with the constants fixed rather than derived from the
//! soundtrack. A drift between the numbers shows up as a visible glitch, so
//! the boundaries are worth pinning.

use super::systems::{
    FALL_END, FALL_START, HOLD_END, INTRO_TOTAL, LOGO_EXIT_Y, LOGO_LAND_Y, LOGO_START_Y,
    intro_position,
};

/// The logo waits off the top of the screen before it starts moving.
#[test]
fn starts_above_the_screen_and_transparent() {
    let (y, alpha) = intro_position(0.0);
    assert_eq!(y, LOGO_START_Y);
    assert_eq!(alpha, 0.0, "it is invisible before the fall");

    let (y, alpha) = intro_position(FALL_START);
    assert_eq!(y, LOGO_START_Y, "still waiting at the start of the fall");
    assert_eq!(alpha, 0.0);
}

#[test]
fn lands_at_the_centre_by_the_end_of_the_fall() {
    let (y, alpha) = intro_position(FALL_END);
    assert_eq!(y, LOGO_LAND_Y);
    assert_eq!(alpha, 1.0, "it is fully opaque once it has landed");
}

/// It sits still between landing and leaving.
#[test]
fn holds_still_in_the_middle() {
    for t in [FALL_END, (FALL_END + HOLD_END) / 2.0, HOLD_END] {
        let (y, alpha) = intro_position(t);
        assert_eq!(y, LOGO_LAND_Y, "moved at t={t}");
        assert_eq!(alpha, 1.0, "faded at t={t}");
    }
}

#[test]
fn leaves_off_the_bottom_by_the_end() {
    let (y, alpha) = intro_position(INTRO_TOTAL);
    assert_eq!(y, LOGO_EXIT_Y, "has not finished leaving at the end");
    assert_eq!(alpha, 1.0);
}

/// The fall accelerates: quadratic ease-in, so the first samples move less than
/// the last ones. A linear or inverted curve here would read as a wrong feel.
#[test]
fn the_fall_eases_in() {
    let (_, early) = intro_position(FALL_START + (FALL_END - FALL_START) * 0.25);
    let (_, late) = intro_position(FALL_START + (FALL_END - FALL_START) * 0.75);
    assert!(late > early, "ease-in should speed up: {early} then {late}",);
}

/// No segment may jump or reverse; a jump is what a bad constant produces.
#[test]
fn motion_is_continuous() {
    const SAMPLES: usize = 2_000;
    let step = INTRO_TOTAL / SAMPLES as f32;
    let (_, mut last_alpha) = intro_position(0.0);
    let mut last_y = LOGO_START_Y;

    for i in 1..=SAMPLES {
        let t = i as f32 * step;
        let (y, alpha) = intro_position(t);
        assert!(y <= last_y + 0.001, "moved up at t={t}: {last_y} -> {y}",);
        assert!(
            (alpha - last_alpha).abs() <= 0.02,
            "alpha jumped at t={t}: {last_alpha} -> {alpha}",
        );
        last_y = y;
        last_alpha = alpha;
    }
}

/// The whole run stays inside the design bounds and a legal alpha.
#[test]
fn stays_in_bounds() {
    for i in 0..=1_000 {
        let t = INTRO_TOTAL * i as f32 / 1_000.0;
        let (y, alpha) = intro_position(t);
        assert!(
            (LOGO_EXIT_Y..=LOGO_START_Y).contains(&y),
            "out of range at t={t}: {y}"
        );
        assert!(
            (0.0..=1.0).contains(&alpha),
            "alpha out of range at t={t}: {alpha}"
        );
    }
}

/// Time past the end must not keep travelling: the timer stops at
/// `INTRO_TOTAL` and the state moves on.
#[test]
fn time_past_the_end_clamps() {
    let (end, _) = intro_position(INTRO_TOTAL);
    for t in [INTRO_TOTAL + 1.0, INTRO_TOTAL + 100.0] {
        let (y, _) = intro_position(t);
        assert_eq!(y, end, "drifted past the end at t={t}");
    }
}

/// The logo accelerates during the fall, so the vertical speed at the very top
/// is legitimately near zero. A jump check therefore has to be relative to how
/// far the logo can move in the sampling interval, not an absolute distance.
#[test]
fn the_segments_hand_over_without_a_jump() {
    // Fastest vertical speed: the linear exit sweep across the fall height.
    let max_speed = (LOGO_START_Y - LOGO_EXIT_Y).abs() / (INTRO_TOTAL - HOLD_END);
    for t in [FALL_START, FALL_END, HOLD_END, INTRO_TOTAL] {
        let (y, _) = intro_position(t);
        let before = intro_position(t - 0.001).0;
        let after = intro_position(t + 0.001).0;
        // Two samples plus the gap between the segments and the easing.
        let budget = max_speed * 0.01 + 1.0;
        assert!(
            (before - y).abs() < budget && (after - y).abs() < budget,
            "position jumps at t={t}: {before} -> {y} -> {after}",
        );
    }
}
