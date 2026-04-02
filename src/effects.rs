use bevy::prelude::*;

use crate::{PixelCamera, PixelShake, PixelSnap, PixelViewportMetrics};

fn hash_noise(seed: u32, time_value: f32, axis: u32) -> f32 {
    let mixed = (seed as f32 * 17.13) + (axis as f32 * 37.77) + time_value * 9.73;
    ((mixed.sin() * 43_758.547).fract() * 2.0) - 1.0
}

pub fn pixel_aligned_shake_offset(shake: &PixelShake, elapsed_seconds: f32) -> IVec2 {
    if shake.amplitude <= 0.0 || shake.frequency <= 0.0 {
        return IVec2::ZERO;
    }

    let sample_time = elapsed_seconds * shake.frequency;
    let offset = Vec2::new(
        hash_noise(shake.seed, sample_time, 0),
        hash_noise(shake.seed.wrapping_add(1), sample_time, 1),
    ) * shake.amplitude;

    IVec2::new(offset.x.round() as i32, offset.y.round() as i32)
}

pub(crate) fn apply_canvas_shake(
    time: Res<Time>,
    mut query: Query<(&mut PixelViewportMetrics, Option<&mut PixelShake>), With<PixelCamera>>,
) {
    let elapsed = time.elapsed_secs();
    let delta = time.delta_secs();

    for (mut metrics, maybe_shake) in &mut query {
        let Some(mut shake) = maybe_shake else {
            metrics.shake_offset = IVec2::ZERO;
            continue;
        };

        metrics.shake_offset = pixel_aligned_shake_offset(&shake, elapsed);
        if shake.decay > 0.0 {
            shake.amplitude = (shake.amplitude - shake.decay * delta).max(0.0);
        }
    }
}

pub(crate) fn snap_marked_entities(mut transforms: Query<&mut Transform, With<PixelSnap>>) {
    for mut transform in &mut transforms {
        transform.translation.x = transform.translation.x.round();
        transform.translation.y = transform.translation.y.round();
    }
}

#[cfg(test)]
#[path = "effects_tests.rs"]
mod tests;
