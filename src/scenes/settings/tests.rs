//! Tests for the settings maths.
//!
//! `fraction_from` converts a pointer hit into a 0..=1 position on the slider
//! track. It is the only place where a screen coordinate becomes a value, so an
//! off-by-half or a missing clamp shows up as a slider that jumps or refuses to
//! reach the ends.

use bevy::prelude::*;

use super::systems::{SliderValue, SoundVolume, ToggleValue, VignetteSettings, fraction_from};

/// A pointer hit arrives in normalized space, spanning -0.5..0.5, so the
/// midpoint of the track is x = 0.
#[test]
fn the_middle_of_the_track_is_the_middle_of_the_press() {
    assert_eq!(fraction_from(Some(Vec3::new(0.0, 0.5, 0.0))), Some(0.5));
}

#[test]
fn the_ends_of_the_track_map_to_zero_and_one() {
    assert_eq!(fraction_from(Some(Vec3::new(-0.5, 0.0, 0.0))), Some(0.0));
    assert_eq!(fraction_from(Some(Vec3::new(0.5, 0.0, 0.0))), Some(1.0));
}

/// Presses that land past the end of the track still clamp, rather than
/// producing a value the track cannot represent.
#[test]
fn out_of_range_presses_clamp() {
    assert_eq!(fraction_from(Some(Vec3::new(-10.0, 0.0, 0.0))), Some(0.0));
    assert_eq!(fraction_from(Some(Vec3::new(10.0, 0.0, 0.0))), Some(1.0));
}

/// The vertical position of the press is ignored; dragging off the track in y
/// must not disturb the value.
#[test]
fn only_the_horizontal_axis_matters() {
    for y in [-100.0, 0.0, 100.0] {
        assert_eq!(
            fraction_from(Some(Vec3::new(0.25, y, 0.0))),
            Some(0.75),
            "y = {y} changed the result",
        );
    }
}

/// A press with no hit position - a click that left the window, say - must not
/// be treated as a position.
#[test]
fn a_press_without_a_position_is_ignored() {
    assert_eq!(fraction_from(None), None);
}

#[test]
fn every_output_is_a_valid_fraction() {
    for i in -50..=50 {
        let x = i as f32 / 50.0;
        let fraction = fraction_from(Some(Vec3::new(x, 0.0, 0.0))).expect("a position");
        assert!((0.0..=1.0).contains(&fraction), "x = {x} gave {fraction}");
    }
}

// ------------------------------------------------------------ value traits

#[test]
fn volume_reports_a_clamped_fraction() {
    let mut volume = SoundVolume(1.0);
    assert_eq!(volume.fraction(), 1.0);
    volume.0 = 0.25;
    assert_eq!(volume.fraction(), 0.25);
    volume.0 = 5.0;
    assert_eq!(
        volume.fraction(),
        1.0,
        "an out-of-range value reads as full"
    );
    volume.0 = -1.0;
    assert_eq!(volume.fraction(), 0.0);
}

#[test]
fn volume_clamps_what_it_is_given() {
    let mut volume = SoundVolume(1.0);
    volume.set_fraction(2.0);
    assert_eq!(volume.0, 1.0);
    volume.set_fraction(-1.0);
    assert_eq!(volume.0, 0.0);
    volume.set_fraction(0.3);
    assert_eq!(volume.0, 0.3);
}

#[test]
fn volume_readout_rounds_to_whole_percent() {
    let mut volume = SoundVolume(1.0);
    assert_eq!(volume.readout(), "100%");
    volume.set_fraction(0.444);
    assert_eq!(volume.readout(), "44%");
    volume.set_fraction(0.445);
    assert_eq!(volume.readout(), "45%");
    volume.set_fraction(0.0);
    assert_eq!(volume.readout(), "0%");
}

#[test]
fn the_vignette_toggles_both_ways() {
    let mut vignette = VignetteSettings::default();
    assert!(vignette.enabled, "on by default");
    vignette.toggle();
    assert!(!vignette.enabled);
    vignette.toggle();
    assert!(vignette.enabled, "toggling twice returns to the start");
}
