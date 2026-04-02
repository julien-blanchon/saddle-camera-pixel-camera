use bevy::{math::UVec2, prelude::*};

use crate::{PixelCamera, PixelViewportMetrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowCanvasInfo {
    pub physical_size: UVec2,
    pub scale_factor: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StaticViewportMetrics {
    pub integer_scale: u32,
    pub zoom: u32,
    pub virtual_size: UVec2,
    pub world_view_size: Vec2,
    pub overscan: UVec2,
    pub render_target_size: UVec2,
    pub window_physical_size: UVec2,
    pub window_logical_size: Vec2,
    pub scale_factor: f64,
    pub viewport_physical_size: UVec2,
    pub viewport_origin_physical: IVec2,
    pub viewport_logical_size: Vec2,
    pub viewport_origin_logical: Vec2,
}

pub fn sanitized_virtual_size(size: UVec2) -> UVec2 {
    UVec2::new(size.x.max(1), size.y.max(1))
}

pub fn sanitized_zoom(zoom: u32) -> u32 {
    zoom.max(1)
}

pub fn compute_integer_scale(window_physical_size: UVec2, virtual_size: UVec2) -> u32 {
    let scale_x = window_physical_size.x / virtual_size.x.max(1);
    let scale_y = window_physical_size.y / virtual_size.y.max(1);
    scale_x.min(scale_y).max(1)
}

pub fn compute_static_metrics(
    camera: &PixelCamera,
    window: WindowCanvasInfo,
) -> StaticViewportMetrics {
    let virtual_size = sanitized_virtual_size(camera.virtual_size);
    let zoom = sanitized_zoom(camera.zoom);
    let integer_scale = compute_integer_scale(window.physical_size, virtual_size);
    let viewport_physical_size = virtual_size * integer_scale;
    let viewport_origin_physical =
        (window.physical_size.as_ivec2() - viewport_physical_size.as_ivec2()) / 2;
    let scale_factor = window.scale_factor.max(1.0);
    let window_logical_size = window.physical_size.as_vec2() / scale_factor as f32;
    let viewport_logical_size = viewport_physical_size.as_vec2() / scale_factor as f32;
    let viewport_origin_logical = viewport_origin_physical.as_vec2() / scale_factor as f32;
    let world_view_size = virtual_size.as_vec2() / zoom as f32;
    let overscan = UVec2::splat(zoom);
    let render_target_size = virtual_size + overscan * 2;

    StaticViewportMetrics {
        integer_scale,
        zoom,
        virtual_size,
        world_view_size,
        overscan,
        render_target_size,
        window_physical_size: window.physical_size,
        window_logical_size,
        scale_factor,
        viewport_physical_size,
        viewport_origin_physical,
        viewport_logical_size,
        viewport_origin_logical,
    }
}

pub fn apply_static_metrics(target: &mut PixelViewportMetrics, metrics: StaticViewportMetrics) {
    target.integer_scale = metrics.integer_scale;
    target.zoom = metrics.zoom;
    target.virtual_size = metrics.virtual_size;
    target.world_view_size = metrics.world_view_size;
    target.overscan = metrics.overscan;
    target.render_target_size = metrics.render_target_size;
    target.window_physical_size = metrics.window_physical_size;
    target.window_logical_size = metrics.window_logical_size;
    target.scale_factor = metrics.scale_factor;
    target.viewport_physical_size = metrics.viewport_physical_size;
    target.viewport_origin_physical = metrics.viewport_origin_physical;
    target.viewport_logical_size = metrics.viewport_logical_size;
    target.viewport_origin_logical = metrics.viewport_origin_logical;
}

pub fn snapped_position(logical_position: Vec2) -> (IVec2, Vec2) {
    let snapped = logical_position.floor();
    let snapped_position = IVec2::new(snapped.x as i32, snapped.y as i32);
    let fractional_offset = logical_position - snapped;
    (snapped_position, fractional_offset)
}

pub fn texture_sample_offset(fractional_offset: Vec2, zoom: u32) -> Vec2 {
    let zoom = sanitized_zoom(zoom) as f32;
    Vec2::new(fractional_offset.x * zoom, -fractional_offset.y * zoom)
}

#[cfg(test)]
#[path = "metrics_tests.rs"]
mod tests;
