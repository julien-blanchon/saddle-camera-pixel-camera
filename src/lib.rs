mod canvas;
mod components;
mod cursor;
mod effects;
mod metrics;
mod subpixel;

pub use components::{
    HIGH_RES_LAYER_INDEX, HIGH_RES_LAYERS, PIXEL_LAYER_INDEX, PIXEL_LAYERS, PixelCamera,
    PixelCameraCanvas, PixelCameraInner, PixelCameraOuter, PixelCameraTransform, PixelCursorHit,
    PixelScaleChanged, PixelShake, PixelSnap, PixelViewportMetrics,
};
pub use cursor::{
    cursor_to_world, screen_to_virtual, screen_to_virtual_physical, screen_to_world,
    screen_to_world_physical,
};
pub use metrics::{
    StaticViewportMetrics, WindowCanvasInfo, compute_integer_scale, compute_static_metrics,
    snapped_position, texture_sample_offset,
};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

use crate::canvas::PixelCameraWindowLayoutVersion;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PixelCameraSystems {
    DetectChanges,
    ComputeMetrics,
    ComputeSubpixel,
    ApplyEffects,
    ApplyTransforms,
    RefitCanvas,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

#[derive(Resource, Default)]
struct PixelCameraRuntimeActive(bool);

pub struct PixelCameraPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl PixelCameraPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for PixelCameraPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for PixelCameraPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.init_resource::<PixelCameraRuntimeActive>()
            .init_resource::<PixelCameraWindowLayoutVersion>()
            .add_message::<PixelScaleChanged>()
            .register_type::<PixelCamera>()
            .register_type::<PixelCameraCanvas>()
            .register_type::<PixelCameraInner>()
            .register_type::<PixelCameraOuter>()
            .register_type::<PixelCameraTransform>()
            .register_type::<PixelShake>()
            .register_type::<PixelSnap>()
            .register_type::<PixelViewportMetrics>()
            .add_systems(self.activate_schedule, activate_runtime)
            .add_systems(
                self.deactivate_schedule,
                (canvas::cleanup_all_roots, deactivate_runtime).chain(),
            )
            .configure_sets(
                self.update_schedule,
                (
                    PixelCameraSystems::DetectChanges,
                    PixelCameraSystems::ComputeMetrics,
                    PixelCameraSystems::ComputeSubpixel,
                    PixelCameraSystems::ApplyEffects,
                    PixelCameraSystems::ApplyTransforms,
                    PixelCameraSystems::RefitCanvas,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                (
                    (canvas::track_window_changes, canvas::cleanup_removed_roots)
                        .chain()
                        .in_set(PixelCameraSystems::DetectChanges),
                    canvas::ensure_pixel_camera_entities.in_set(PixelCameraSystems::ComputeMetrics),
                    subpixel::compute_subpixel_offset.in_set(PixelCameraSystems::ComputeSubpixel),
                    (effects::apply_canvas_shake, effects::snap_marked_entities)
                        .chain()
                        .in_set(PixelCameraSystems::ApplyEffects),
                    (
                        subpixel::apply_inner_camera_transform,
                        subpixel::apply_canvas_transform,
                    )
                        .chain()
                        .in_set(PixelCameraSystems::ApplyTransforms),
                    canvas::refit_canvas_presentation.in_set(PixelCameraSystems::RefitCanvas),
                )
                    .run_if(runtime_is_active),
            );
    }
}

fn activate_runtime(mut runtime: ResMut<PixelCameraRuntimeActive>) {
    runtime.0 = true;
}

fn deactivate_runtime(mut runtime: ResMut<PixelCameraRuntimeActive>) {
    runtime.0 = false;
}

fn runtime_is_active(runtime: Res<PixelCameraRuntimeActive>) -> bool {
    runtime.0
}

#[cfg(test)]
#[path = "integration_tests.rs"]
mod integration_tests;
