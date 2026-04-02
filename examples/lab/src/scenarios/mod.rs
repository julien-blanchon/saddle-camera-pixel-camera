use bevy::{prelude::*, window::PrimaryWindow};
use bevy_e2e::{
    action::Action,
    actions::{assertions, inspect},
    scenario::Scenario,
};
use saddle_camera_pixel_camera::{
    PixelCamera, PixelCameraCanvas, PixelCameraInner, PixelCameraOuter, PixelCameraTransform,
    PixelViewportMetrics, screen_to_world,
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
            world.resource_mut::<LabCameraMotion>().enabled = true;
            world.resource_mut::<LabCameraMotion>().velocity = Vec2::new(19.0, 0.0);
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
            world.resource_mut::<LabCameraMotion>().enabled = false;
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
            let mut query = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
            let mut window = query.single_mut(world).expect("primary window exists");
            window.resolution.set_physical_resolution(1024, 768);
            window.resolution.set_scale_factor_override(Some(1.0));
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
            let mut query = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
            let mut window = query.single_mut(world).expect("primary window exists");
            window.resolution.set_scale_factor_override(Some(2.0));
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
            let root = root(world);
            let mut entity = world.entity_mut(root);
            let mut camera = entity.get_mut::<PixelCamera>().expect("pixel camera exists");
            camera.zoom = 3;
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
            let mut query = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
            let mut window = query.single_mut(world).expect("primary window exists");
            window.resolution.set_physical_resolution(1024, 768);
            window.resolution.set_scale_factor_override(Some(1.0));
        })))
        .then(Action::WaitFrames(8))
        .then(Action::Custom(Box::new(|world| {
            let viewport = metrics(world);
            let mut query = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
            let mut window = query.single_mut(world).expect("primary window exists");
            let letterbox_position = Vec2::new(
                viewport.viewport_origin_logical.x * 0.5,
                viewport.viewport_origin_logical.y * 0.5,
            );
            window.set_cursor_position(Some(letterbox_position));
        })))
        .then(Action::WaitFrames(4))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "letterbox cursor position returns none",
            |diagnostics| diagnostics.last_cursor_hit.is_none(),
        ))
        .then(Action::Screenshot("cursor_letterbox".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world| {
            let viewport = metrics(world);
            let mut query = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
            let mut window = query.single_mut(world).expect("primary window exists");
            window.set_cursor_position(Some(
                viewport.viewport_origin_logical + viewport.viewport_logical_size * 0.5,
            ));
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
