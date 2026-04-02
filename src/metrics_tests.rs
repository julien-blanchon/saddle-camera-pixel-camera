use super::*;

fn window(width: u32, height: u32, scale_factor: f64) -> WindowCanvasInfo {
    WindowCanvasInfo {
        physical_size: UVec2::new(width, height),
        scale_factor,
    }
}

#[test]
fn integer_scale_exact_fit() {
    assert_eq!(
        compute_integer_scale(UVec2::new(1280, 720), UVec2::new(320, 180)),
        4
    );
}

#[test]
fn integer_scale_aspect_mismatch() {
    assert_eq!(
        compute_integer_scale(UVec2::new(1200, 720), UVec2::new(320, 180)),
        3
    );
}

#[test]
fn integer_scale_clamps_to_one_for_tiny_windows() {
    assert_eq!(
        compute_integer_scale(UVec2::new(200, 100), UVec2::new(320, 180)),
        1
    );
}

#[test]
fn viewport_origin_centers_letterboxed_canvas() {
    let camera = PixelCamera::new(320, 180);
    let metrics = compute_static_metrics(&camera, window(1920, 1200, 1.0));
    assert_eq!(metrics.integer_scale, 6);
    assert_eq!(metrics.viewport_physical_size, UVec2::new(1920, 1080));
    assert_eq!(metrics.viewport_origin_physical, IVec2::new(0, 60));
}

#[test]
fn render_target_size_grows_with_zoom_overscan() {
    let mut camera = PixelCamera::new(320, 180);
    camera.zoom = 3;
    let metrics = compute_static_metrics(&camera, window(1920, 1080, 1.0));
    assert_eq!(metrics.overscan, UVec2::splat(3));
    assert_eq!(metrics.render_target_size, UVec2::new(326, 186));
}

#[test]
fn dpi_aware_logical_metrics_use_scale_factor() {
    let camera = PixelCamera::new(320, 180);
    let metrics = compute_static_metrics(&camera, window(1920, 1080, 2.0));
    assert_eq!(metrics.viewport_logical_size, Vec2::new(960.0, 540.0));
    assert_eq!(metrics.viewport_origin_logical, Vec2::ZERO);
    assert_eq!(metrics.window_logical_size, Vec2::new(960.0, 540.0));
}

#[test]
fn zoom_changes_world_view_but_not_integer_scale() {
    let base = PixelCamera::new(320, 180);
    let mut zoomed = PixelCamera::new(320, 180);
    zoomed.zoom = 4;

    let base_metrics = compute_static_metrics(&base, window(1280, 720, 1.0));
    let zoomed_metrics = compute_static_metrics(&zoomed, window(1280, 720, 1.0));

    assert_eq!(base_metrics.integer_scale, zoomed_metrics.integer_scale);
    assert_eq!(zoomed_metrics.world_view_size, Vec2::new(80.0, 45.0));
}
