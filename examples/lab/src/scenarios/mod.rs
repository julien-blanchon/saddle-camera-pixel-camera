use bevy::{prelude::*, window::PrimaryWindow};
use saddle_bevy_e2e::{
    action::Action,
    actions::{assertions, inspect},
    scenario::Scenario,
};
use saddle_camera_pixel_camera::{
    PixelCamera, PixelCameraCanvas, PixelCameraInner, PixelCameraOuter, PixelCameraScaleMode,
    PixelCameraTransform, PixelViewportMetrics, screen_to_world,
};

use crate::{LabCameraMotion, LabDiagnostics, LabRoot, support};

#[derive(Resource, Clone, Copy)]
struct Snapshot {
    snapped_position: IVec2,
}

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "pixel_camera_smoke",
        "pixel_camera_subpixel_scroll",
        "pixel_camera_resize",
        "pixel_camera_zoom",
        "pixel_camera_scale_modes",
        "pixel_camera_mixed_layers",
        "pixel_camera_cursor",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "pixel_camera_smoke" => Some(build_smoke()),
        "pixel_camera_subpixel_scroll" => Some(build_subpixel_scroll()),
        "pixel_camera_resize" => Some(build_resize()),
        "pixel_camera_zoom" => Some(build_zoom()),
        "pixel_camera_scale_modes" => Some(build_scale_modes()),
        "pixel_camera_mixed_layers" => Some(build_mixed_layers()),
        "pixel_camera_cursor" => Some(build_cursor()),
        _ => None,
    }
}

fn root(world: &World) -> Entity {
    world.resource::<LabRoot>().0
}

fn metrics(world: &World) -> PixelViewportMetrics {
    world
        .get::<PixelViewportMetrics>(root(world))
        .expect("pixel camera metrics should exist")
        .clone()
}

fn resize_primary_window(
    world: &mut World,
    physical_width: u32,
    physical_height: u32,
    scale_factor_override: Option<f32>,
) {
    let mut query = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
    let mut window = query.single_mut(world).expect("primary window exists");
    window
        .resolution
        .set_physical_resolution(physical_width, physical_height);
    window.resolution.set_scale_factor_override(scale_factor_override);
}

fn set_cursor_position(world: &mut World, position: Vec2) {
    let mut query = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
    let mut window = query.single_mut(world).expect("primary window exists");
    window.set_cursor_position(Some(position));
}

fn set_scale_mode_index(world: &mut World, index: f32) {
    world.resource_mut::<support::ExamplePixelPane>().scale_mode_index = index;
}

fn apply_gameboy_preset(world: &mut World) {
    let preset = PixelCamera::gameboy();
    let mut pane = world.resource_mut::<support::ExamplePixelPane>();
    pane.virtual_width = preset.virtual_size.x as f32;
    pane.virtual_height = preset.virtual_size.y as f32;
    pane.zoom = 2.0;
    pane.scale_mode_index = match preset.scale_mode {
        PixelCameraScaleMode::IntegerLetterbox => 0.0,
        PixelCameraScaleMode::IntegerCrop => 1.0,
        PixelCameraScaleMode::FractionalFit => 2.0,
    };
}

fn set_camera_motion(world: &mut World, enabled: bool, velocity: Vec2) {
    let mut motion = world.resource_mut::<LabCameraMotion>();
    motion.enabled = enabled;
    motion.velocity = velocity;
}

fn set_camera_zoom(world: &mut World, zoom: u32) {
    let root = root(world);
    let mut entity = world.entity_mut(root);
    let mut camera = entity.get_mut::<PixelCamera>().expect("pixel camera exists");
    camera.zoom = zoom;
}

fn letterbox_cursor_position(world: &World) -> Vec2 {
    let viewport = metrics(world);
    Vec2::new(
        viewport.viewport_origin_logical.x * 0.5,
        viewport.viewport_origin_logical.y * 0.5,
    )
}

fn viewport_center_cursor_position(world: &World) -> Vec2 {
    let viewport = metrics(world);
    viewport.viewport_origin_logical + viewport.viewport_logical_size * 0.5
}

fn build_smoke() -> Scenario {
    Scenario::builder("pixel_camera_smoke")
        .description(
            "Boot the lab, confirm the root plus helper entities exist, and capture a clean baseline frame.",
        )
        .then(Action::WaitFrames(45))
        .then(assertions::entity_exists::<PixelCamera>("pixel camera root exists"))
        .then(assertions::entity_count::<PixelCameraInner>("one inner camera", 1))
        .then(assertions::entity_count::<PixelCameraCanvas>("one canvas sprite", 1))
        .then(assertions::entity_count::<PixelCameraOuter>("one outer camera", 1))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "cursor diagnostics resource exists",
            |_| true,
        ))
        .then(assertions::custom("integer scale is non-zero", |world| {
            metrics(world).integer_scale >= 1
        }))
        .then(inspect::dump_component_json::<PixelViewportMetrics>("smoke_metrics"))
        .then(assertions::log_summary("pixel_camera_smoke summary"))
        .then(Action::Screenshot("pixel_camera_smoke".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_subpixel_scroll() -> Scenario {
    Scenario::builder("pixel_camera_subpixel_scroll")
        .description(
            "Drive the logical camera at a non-integer speed, assert the snapped position changes and the fractional remainder stays active, then capture multiple checkpoints.",
        )
        .then(Action::WaitFrames(30))
        .then(Action::Custom(Box::new(|world| {
            world.insert_resource(Snapshot {
                snapped_position: metrics(world).snapped_position,
            });
            set_camera_motion(world, true, Vec2::new(19.0, 0.0));
        })))
        .then(Action::Screenshot("subpixel_scroll_start".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(75))
        .then(assertions::custom("fractional remainder becomes non-zero", |world| {
            metrics(world).fractional_offset.length() > 0.01
        }))
        .then(Action::Screenshot("subpixel_scroll_mid".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(75))
        .then(assertions::custom("snapped position advances", |world| {
            metrics(world).snapped_position != world.resource::<Snapshot>().snapped_position
        }))
        .then(Action::Custom(Box::new(|world| {
            set_camera_motion(world, false, Vec2::ZERO);
        })))
        .then(Action::Screenshot("subpixel_scroll_end".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("pixel_camera_subpixel_scroll summary"))
        .build()
}

fn build_resize() -> Scenario {
    Scenario::builder("pixel_camera_resize")
        .description(
            "Resize to an aspect-mismatched window to force letterboxing, then apply a DPI override and assert both physical and logical viewport metrics update correctly.",
        )
        .then(Action::WaitFrames(30))
        .then(Action::Screenshot("resize_before".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world| {
            resize_primary_window(world, 1024, 768, Some(1.0));
        })))
        .then(Action::WaitFrames(8))
        .then(assertions::custom(
            "aspect-mismatched resize produces letterboxing",
            |world| {
                let metrics = metrics(world);
                (metrics.viewport_origin_physical.x > 0 || metrics.viewport_origin_physical.y > 0)
                    && (metrics.viewport_physical_size.x < metrics.window_physical_size.x
                        || metrics.viewport_physical_size.y < metrics.window_physical_size.y)
                    && metrics.viewport_physical_size == metrics.virtual_size * metrics.integer_scale
            },
        ))
        .then(Action::Screenshot("resize_letterboxed".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world| {
            resize_primary_window(world, 1024, 768, Some(2.0));
        })))
        .then(Action::WaitFrames(8))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "scale change message observed",
            |diagnostics| diagnostics.scale_events > 0,
        ))
        .then(assertions::custom("dpi override updates logical viewport metrics", |world| {
            let metrics = metrics(world);
            metrics.scale_factor >= 2.0
                && metrics.integer_scale >= 1
                && metrics.viewport_logical_size.distance(
                    metrics.viewport_physical_size.as_vec2() / metrics.scale_factor as f32,
                ) < 0.01
                && metrics.viewport_origin_logical.distance(
                    metrics.viewport_origin_physical.as_vec2() / metrics.scale_factor as f32,
                ) < 0.01
        }))
        .then(Action::Screenshot("resize_dpi_after".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("pixel_camera_resize summary"))
        .build()
}

fn build_zoom() -> Scenario {
    Scenario::builder("pixel_camera_zoom")
        .description(
            "Change integer zoom at runtime, assert the public metrics update, and capture before/after frames.",
        )
        .then(Action::WaitFrames(30))
        .then(Action::Screenshot("zoom_before".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world| {
            set_camera_zoom(world, 3);
        })))
        .then(Action::WaitFrames(8))
        .then(assertions::custom("zoom updates metrics", |world| {
            let metrics = metrics(world);
            metrics.zoom == 3 && metrics.render_target_size == UVec2::new(326, 186)
        }))
        .then(Action::Screenshot("zoom_after".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("pixel_camera_zoom summary"))
        .build()
}

fn build_scale_modes() -> Scenario {
    Scenario::builder("pixel_camera_scale_modes")
        .description(
            "Force an awkward window size, verify integer-crop and fractional-fit presentation modes, then switch to a retro preset and assert the metrics rebuild around the new virtual resolution.",
        )
        .then(Action::WaitFrames(30))
        .then(Action::Custom(Box::new(|world| {
            resize_primary_window(world, 1024, 768, Some(1.0));
        })))
        .then(Action::WaitFrames(8))
        .then(Action::Custom(Box::new(|world| {
            set_scale_mode_index(world, 1.0);
        })))
        .then(Action::WaitFrames(8))
        .then(assertions::custom(
            "integer crop fills one axis and overflows the other",
            |world| {
                let metrics = metrics(world);
                metrics.presentation_scale == metrics.integer_scale as f32
                    && metrics.viewport_physical_size.x >= metrics.window_physical_size.x
                    && metrics.viewport_physical_size.y >= metrics.window_physical_size.y
                    && (metrics.viewport_physical_size.x > metrics.window_physical_size.x
                        || metrics.viewport_physical_size.y > metrics.window_physical_size.y)
                    && metrics.viewport_origin_physical.x <= 0
                    && metrics.viewport_origin_physical.y <= 0
                    && (metrics.viewport_origin_physical.x < 0
                        || metrics.viewport_origin_physical.y < 0)
            },
        ))
        .then(Action::Screenshot("scale_modes_crop".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world| {
            set_scale_mode_index(world, 2.0);
        })))
        .then(Action::WaitFrames(8))
        .then(assertions::custom(
            "fractional fit keeps a non-integer presentation scale",
            |world| {
                let metrics = metrics(world);
                metrics.presentation_scale > metrics.integer_scale as f32
                    && (metrics.presentation_scale - metrics.presentation_scale.round()).abs() > 0.01
            },
        ))
        .then(Action::Screenshot("scale_modes_fractional_fit".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world| {
            apply_gameboy_preset(world);
        })))
        .then(Action::WaitFrames(8))
        .then(assertions::custom("retro preset updates the virtual resolution", |world| {
            let metrics = metrics(world);
            metrics.virtual_size == UVec2::new(160, 144) && metrics.zoom == 2
        }))
        .then(assertions::log_summary("pixel_camera_scale_modes summary"))
        .then(Action::Screenshot("scale_modes_gameboy".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_mixed_layers() -> Scenario {
    Scenario::builder("pixel_camera_mixed_layers")
        .description(
            "Verify the pixel-world root and high-resolution overlay diagnostics coexist in one frame.",
        )
        .then(Action::WaitFrames(30))
        .then(assertions::entity_exists::<PixelCameraCanvas>("canvas sprite exists"))
        .then(assertions::entity_exists::<support::DemoActor>("pixel actor exists"))
        .then(assertions::entity_exists::<support::HighResBadge>("high-res badge exists"))
        .then(Action::Screenshot("mixed_layers".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("pixel_camera_mixed_layers summary"))
        .build()
}

fn build_cursor() -> Scenario {
    Scenario::builder("pixel_camera_cursor")
        .description(
            "Force an aspect-mismatched viewport, prove letterbox cursor positions return none, then map the canvas center back to the logical camera center.",
        )
        .then(Action::WaitFrames(20))
        .then(Action::Custom(Box::new(|world| {
            resize_primary_window(world, 1024, 768, Some(1.0));
        })))
        .then(Action::WaitFrames(8))
        .then(Action::Custom(Box::new(|world| {
            let position = letterbox_cursor_position(world);
            set_cursor_position(world, position);
        })))
        .then(Action::WaitFrames(4))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "letterbox cursor position returns none",
            |diagnostics| diagnostics.last_cursor_hit.is_none(),
        ))
        .then(Action::Screenshot("cursor_letterbox".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world| {
            let position = viewport_center_cursor_position(world);
            set_cursor_position(world, position);
        })))
        .then(Action::WaitFrames(4))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "cursor tracking produced a hit",
            |diagnostics| diagnostics.last_cursor_hit.is_some(),
        ))
        .then(Action::Custom(Box::new(|world| {
            let viewport = metrics(world);
            let camera = world
                .get::<PixelCameraTransform>(root(world))
                .expect("camera transform exists")
                .clone();
            let mut window_query = world.query_filtered::<&Window, With<PrimaryWindow>>();
            let window = window_query.single(world).expect("primary window exists");
            let hit = screen_to_world(
                window
                    .cursor_position()
                    .expect("cursor should be inside window"),
                &viewport,
                &camera,
            )
            .expect("cursor should map into pixel viewport");
            assert!(
                hit.world_position.distance(camera.logical_position) < 1.0,
                "cursor mapping should hit near the logical camera center"
            );
        })))
        .then(Action::Screenshot("cursor_center".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("pixel_camera_cursor summary"))
        .build()
}
