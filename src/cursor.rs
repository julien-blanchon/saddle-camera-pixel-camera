use bevy::{prelude::*, window::Window};

use crate::{PixelCameraTransform, PixelCursorHit, PixelViewportMetrics};

fn clamp_virtual_pixel(virtual_position: Vec2, virtual_size: UVec2) -> UVec2 {
    UVec2::new(
        virtual_position
            .x
            .floor()
            .clamp(0.0, virtual_size.x.saturating_sub(1) as f32) as u32,
        virtual_position
            .y
            .floor()
            .clamp(0.0, virtual_size.y.saturating_sub(1) as f32) as u32,
    )
}

pub fn screen_to_virtual(
    screen_position_logical: Vec2,
    metrics: &PixelViewportMetrics,
) -> Option<Vec2> {
    if !metrics.contains_logical_point(screen_position_logical) {
        return None;
    }

    let scale_factor = metrics.scale_factor.max(1.0) as f32;
    let relative_logical = screen_position_logical - metrics.viewport_origin_logical;
    let relative_physical = relative_logical * scale_factor;
    Some(relative_physical / metrics.integer_scale.max(1) as f32)
}

pub fn screen_to_virtual_physical(
    screen_position_physical: Vec2,
    metrics: &PixelViewportMetrics,
) -> Option<Vec2> {
    if !metrics.contains_physical_point(screen_position_physical) {
        return None;
    }

    let relative_physical = screen_position_physical - metrics.viewport_origin_physical.as_vec2();
    Some(relative_physical / metrics.integer_scale.max(1) as f32)
}

pub fn screen_to_world(
    screen_position_logical: Vec2,
    metrics: &PixelViewportMetrics,
    camera: &PixelCameraTransform,
) -> Option<PixelCursorHit> {
    let virtual_position = screen_to_virtual(screen_position_logical, metrics)?;
    let top_left_world = camera.logical_position
        + Vec2::new(
            -metrics.world_view_size.x * 0.5,
            metrics.world_view_size.y * 0.5,
        );
    let world_position = top_left_world
        + Vec2::new(
            virtual_position.x / metrics.zoom.max(1) as f32,
            -virtual_position.y / metrics.zoom.max(1) as f32,
        );

    Some(PixelCursorHit {
        virtual_pixel: clamp_virtual_pixel(virtual_position, metrics.virtual_size),
        virtual_position,
        world_position,
    })
}

pub fn screen_to_world_physical(
    screen_position_physical: Vec2,
    metrics: &PixelViewportMetrics,
    camera: &PixelCameraTransform,
) -> Option<PixelCursorHit> {
    let virtual_position = screen_to_virtual_physical(screen_position_physical, metrics)?;
    let top_left_world = camera.logical_position
        + Vec2::new(
            -metrics.world_view_size.x * 0.5,
            metrics.world_view_size.y * 0.5,
        );
    let world_position = top_left_world
        + Vec2::new(
            virtual_position.x / metrics.zoom.max(1) as f32,
            -virtual_position.y / metrics.zoom.max(1) as f32,
        );

    Some(PixelCursorHit {
        virtual_pixel: clamp_virtual_pixel(virtual_position, metrics.virtual_size),
        virtual_position,
        world_position,
    })
}

pub fn cursor_to_world(
    window: &Window,
    metrics: &PixelViewportMetrics,
    camera: &PixelCameraTransform,
) -> Option<PixelCursorHit> {
    let cursor = window.cursor_position()?;
    screen_to_world(cursor, metrics, camera)
}

#[cfg(test)]
#[path = "cursor_tests.rs"]
mod tests;
