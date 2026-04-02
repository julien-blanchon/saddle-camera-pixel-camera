use bevy::prelude::*;

use crate::{
    PixelCamera, PixelCameraCanvas, PixelCameraInner, PixelCameraTransform, PixelViewportMetrics,
    components::PixelCameraChildren,
    metrics::{snapped_position, texture_sample_offset},
};

pub(crate) fn compute_subpixel_offset(
    mut roots: Query<(&PixelCameraTransform, &mut PixelViewportMetrics), With<PixelCamera>>,
) {
    for (camera_transform, mut metrics) in &mut roots {
        let (snapped, fractional) = snapped_position(camera_transform.logical_position);
        metrics.snapped_position = snapped;
        metrics.fractional_offset = fractional;
        metrics.texture_sample_offset = texture_sample_offset(fractional, metrics.zoom);
    }
}

pub(crate) fn apply_inner_camera_transform(
    roots: Query<(&PixelViewportMetrics, &PixelCameraChildren), With<PixelCamera>>,
    mut inner_cameras: Query<&mut Transform, With<PixelCameraInner>>,
) {
    for (metrics, children) in &roots {
        let Ok(mut transform) = inner_cameras.get_mut(children.inner) else {
            continue;
        };

        transform.translation.x = metrics.snapped_position.x as f32;
        transform.translation.y = metrics.snapped_position.y as f32;
    }
}

pub(crate) fn apply_canvas_transform(
    roots: Query<(&PixelViewportMetrics, &PixelCameraChildren), With<PixelCamera>>,
    mut canvases: Query<&mut Transform, With<PixelCameraCanvas>>,
) {
    for (metrics, children) in &roots {
        let Ok(mut transform) = canvases.get_mut(children.canvas) else {
            continue;
        };

        let viewport_center = metrics.viewport_origin_logical + metrics.viewport_logical_size * 0.5;
        let shake_logical = metrics.shake_offset.as_vec2() * metrics.integer_scale as f32
            / metrics.scale_factor as f32;

        transform.translation.x =
            viewport_center.x - metrics.window_logical_size.x * 0.5 + shake_logical.x;
        transform.translation.y =
            metrics.window_logical_size.y * 0.5 - viewport_center.y - shake_logical.y;
    }
}

#[cfg(test)]
#[path = "subpixel_tests.rs"]
mod tests;
