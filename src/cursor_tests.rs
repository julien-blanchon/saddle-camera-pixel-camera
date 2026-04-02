use super::*;

fn metrics() -> PixelViewportMetrics {
    PixelViewportMetrics {
        integer_scale: 4,
        zoom: 2,
        virtual_size: UVec2::new(320, 180),
        world_view_size: Vec2::new(160.0, 90.0),
        overscan: UVec2::splat(2),
        render_target_size: UVec2::new(324, 184),
        window_physical_size: UVec2::new(1600, 900),
        window_logical_size: Vec2::new(1600.0, 900.0),
        scale_factor: 1.0,
        viewport_physical_size: UVec2::new(1280, 720),
        viewport_origin_physical: IVec2::new(160, 90),
        viewport_logical_size: Vec2::new(1280.0, 720.0),
        viewport_origin_logical: Vec2::new(160.0, 90.0),
        ..default()
    }
}

#[test]
fn center_of_viewport_maps_to_virtual_center() {
    let metrics = metrics();
    let virtual_position =
        screen_to_virtual(Vec2::new(800.0, 450.0), &metrics).expect("inside viewport");
    assert_eq!(virtual_position, Vec2::new(160.0, 90.0));
}

#[test]
fn letterboxed_area_returns_none() {
    let metrics = metrics();
    assert!(screen_to_virtual(Vec2::new(80.0, 45.0), &metrics).is_none());
}

#[test]
fn physical_center_of_viewport_maps_to_virtual_center() {
    let metrics = metrics();
    let virtual_position =
        screen_to_virtual_physical(Vec2::new(800.0, 450.0), &metrics).expect("inside viewport");
    assert_eq!(virtual_position, Vec2::new(160.0, 90.0));
}

#[test]
fn physical_letterboxed_area_returns_none() {
    let metrics = metrics();
    assert!(screen_to_virtual_physical(Vec2::new(80.0, 45.0), &metrics).is_none());
}

#[test]
fn world_mapping_respects_zoom_and_top_left_origin() {
    let metrics = metrics();
    let camera = PixelCameraTransform {
        logical_position: Vec2::new(40.0, 10.0),
    };
    let hit = screen_to_world(Vec2::new(800.0, 450.0), &metrics, &camera).expect("inside viewport");
    assert_eq!(hit.virtual_position, Vec2::new(160.0, 90.0));
    assert_eq!(hit.world_position, Vec2::new(40.0, 10.0));
}
