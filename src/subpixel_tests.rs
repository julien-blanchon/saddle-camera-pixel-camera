use super::*;

#[test]
fn exact_integer_position_has_zero_fractional_offset() {
    let (snapped, fractional) = snapped_position(Vec2::new(12.0, -4.0));
    assert_eq!(snapped, IVec2::new(12, -4));
    assert_eq!(fractional, Vec2::ZERO);
}

#[test]
fn positive_fractional_position_uses_floor_not_round() {
    let (snapped, fractional) = snapped_position(Vec2::new(8.75, 3.25));
    assert_eq!(snapped, IVec2::new(8, 3));
    assert_eq!(fractional, Vec2::new(0.75, 0.25));
}

#[test]
fn negative_fractional_position_uses_floor() {
    let (snapped, fractional) = snapped_position(Vec2::new(-5.4, -1.1));
    assert_eq!(snapped, IVec2::new(-6, -2));
    assert!((fractional.x - 0.6).abs() < 0.0001);
    assert!((fractional.y - 0.9).abs() < 0.0001);
}

#[test]
fn texture_offset_scales_with_zoom() {
    assert_eq!(
        texture_sample_offset(Vec2::new(0.5, 0.25), 3),
        Vec2::new(1.5, -0.75)
    );
}

#[test]
fn near_boundary_values_stay_stable() {
    let (snapped, fractional) = snapped_position(Vec2::new(0.9999, 1.0001));
    assert_eq!(snapped, IVec2::new(0, 1));
    assert!((fractional.x - 0.9999).abs() < 0.0001);
    assert!((fractional.y - 0.0001).abs() < 0.0001);
}

#[test]
fn large_coordinates_preserve_fractional_component() {
    let (snapped, fractional) = snapped_position(Vec2::new(10_000.375, -20_000.625));
    assert_eq!(snapped, IVec2::new(10_000, -20_001));
    assert!((fractional.x - 0.375).abs() < 0.0001);
    assert!((fractional.y - 0.375).abs() < 0.0001);
}
