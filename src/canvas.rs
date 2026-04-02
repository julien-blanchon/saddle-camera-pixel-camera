use bevy::{
    asset::RenderAssetUsages,
    camera::{
        Camera, Camera2d, ClearColorConfig, OrthographicProjection, Projection, RenderTarget,
        ScalingMode, visibility::RenderLayers,
    },
    ecs::message::MessageReader,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    window::{PrimaryWindow, Window, WindowResized, WindowScaleFactorChanged},
};

use crate::{
    PixelCamera, PixelCameraCanvas, PixelCameraInner, PixelCameraOuter, PixelScaleChanged,
    PixelViewportMetrics,
    components::{PixelCameraChildren, PixelCameraWindowVersion},
    metrics::{
        StaticViewportMetrics, WindowCanvasInfo, apply_static_metrics, compute_static_metrics,
    },
};

#[derive(Resource, Default)]
pub(crate) struct PixelCameraWindowLayoutVersion(pub u64);

fn primary_window_info(window: &Window) -> WindowCanvasInfo {
    WindowCanvasInfo {
        physical_size: UVec2::new(window.physical_width(), window.physical_height()),
        scale_factor: window.scale_factor() as f64,
    }
}

fn build_render_target_image(size: UVec2) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: size.x.max(1),
            height: size.y.max(1),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    image
}

fn crop_rect(metrics: &StaticViewportMetrics, texture_offset: Vec2) -> Rect {
    let min = metrics.overscan.as_vec2() + texture_offset;
    Rect::from_corners(min, min + metrics.virtual_size.as_vec2())
}

fn spawn_helpers(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    root: Entity,
    camera: &PixelCamera,
    metrics: StaticViewportMetrics,
) -> PixelCameraChildren {
    let image = images.add(build_render_target_image(metrics.render_target_size));
    let inner = commands
        .spawn((
            Name::new("Pixel Camera Inner"),
            Camera2d,
            Camera {
                order: camera.outer_camera_order.saturating_sub(1),
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::Fixed {
                    width: metrics.world_view_size.x + 2.0,
                    height: metrics.world_view_size.y + 2.0,
                },
                ..OrthographicProjection::default_2d()
            }),
            RenderTarget::Image(image.clone().into()),
            Msaa::Off,
            camera.world_layers.clone(),
            PixelCameraInner { root },
        ))
        .id();

    let canvas = commands
        .spawn((
            Name::new("Pixel Camera Canvas"),
            Sprite {
                image: image.clone(),
                custom_size: Some(metrics.viewport_logical_size),
                rect: Some(crop_rect(&metrics, Vec2::ZERO)),
                ..default()
            },
            Transform::default(),
            camera.high_res_layers.clone(),
            PixelCameraCanvas { root },
        ))
        .id();

    let outer = commands
        .spawn((
            Name::new("Pixel Camera Outer"),
            Camera2d,
            Camera {
                order: camera.outer_camera_order,
                clear_color: ClearColorConfig::Custom(camera.letterbox_color),
                ..default()
            },
            Msaa::Off,
            camera.high_res_layers.clone(),
            PixelCameraOuter { root },
        ))
        .id();

    commands.entity(root).add_children(&[inner, canvas, outer]);

    PixelCameraChildren {
        inner,
        canvas,
        outer,
        image,
    }
}

fn reconfigure_inner_camera(
    inner: &mut Query<
        (
            &mut Camera,
            &mut Projection,
            &mut RenderTarget,
            &mut RenderLayers,
        ),
        (
            With<PixelCameraInner>,
            Without<PixelCameraCanvas>,
            Without<PixelCameraOuter>,
        ),
    >,
    child: Entity,
    camera: &PixelCamera,
    metrics: StaticViewportMetrics,
    image: Handle<Image>,
) {
    let Ok((mut inner_camera, mut projection, mut render_target, mut layers)) =
        inner.get_mut(child)
    else {
        return;
    };

    inner_camera.order = camera.outer_camera_order.saturating_sub(1);
    *layers = camera.world_layers.clone();
    *render_target = RenderTarget::Image(image.into());
    *projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::Fixed {
            width: metrics.world_view_size.x + 2.0,
            height: metrics.world_view_size.y + 2.0,
        },
        ..OrthographicProjection::default_2d()
    });
}

fn reconfigure_canvas_sprite(
    canvases: &mut Query<
        (&mut Sprite, &mut RenderLayers),
        (
            With<PixelCameraCanvas>,
            Without<PixelCameraInner>,
            Without<PixelCameraOuter>,
        ),
    >,
    child: Entity,
    camera: &PixelCamera,
    metrics: StaticViewportMetrics,
    image: Handle<Image>,
) {
    let Ok((mut sprite, mut layers)) = canvases.get_mut(child) else {
        return;
    };

    sprite.image = image;
    sprite.custom_size = Some(metrics.viewport_logical_size);
    sprite.rect = Some(crop_rect(&metrics, Vec2::ZERO));
    *layers = camera.high_res_layers.clone();
}

fn reconfigure_outer_camera(
    outers: &mut Query<
        (&mut Camera, &mut RenderLayers),
        (
            With<PixelCameraOuter>,
            Without<PixelCameraInner>,
            Without<PixelCameraCanvas>,
        ),
    >,
    child: Entity,
    camera: &PixelCamera,
) {
    let Ok((mut outer_camera, mut layers)) = outers.get_mut(child) else {
        return;
    };

    outer_camera.order = camera.outer_camera_order;
    outer_camera.clear_color = ClearColorConfig::Custom(camera.letterbox_color);
    *layers = camera.high_res_layers.clone();
}

fn despawn_child_if_present(commands: &mut Commands, entity: Entity) {
    if let Ok(mut entity_commands) = commands.get_entity(entity) {
        entity_commands.despawn();
    }
}

fn helpers_missing(
    children: &PixelCameraChildren,
    inner_cameras: &Query<(), With<PixelCameraInner>>,
    canvas_sprites: &Query<(), With<PixelCameraCanvas>>,
    outer_cameras: &Query<(), With<PixelCameraOuter>>,
) -> bool {
    inner_cameras.get(children.inner).is_err()
        || canvas_sprites.get(children.canvas).is_err()
        || outer_cameras.get(children.outer).is_err()
}

fn cleanup_helper_entities(commands: &mut Commands, children: &PixelCameraChildren) {
    despawn_child_if_present(commands, children.inner);
    despawn_child_if_present(commands, children.canvas);
    despawn_child_if_present(commands, children.outer);
}

pub(crate) fn track_window_changes(
    mut version: ResMut<PixelCameraWindowLayoutVersion>,
    mut resized: MessageReader<WindowResized>,
    mut scale_changed: MessageReader<WindowScaleFactorChanged>,
) {
    let resized_changed = resized.read().next().is_some();
    let scale_changed = scale_changed.read().next().is_some();

    if resized_changed || scale_changed {
        version.0 = version.0.wrapping_add(1);
    }
}

pub(crate) fn cleanup_removed_roots(
    mut commands: Commands,
    mut removed: RemovedComponents<PixelCamera>,
    children: Query<&PixelCameraChildren>,
) {
    for root in removed.read() {
        if let Ok(child_refs) = children.get(root) {
            despawn_child_if_present(&mut commands, child_refs.inner);
            despawn_child_if_present(&mut commands, child_refs.canvas);
            despawn_child_if_present(&mut commands, child_refs.outer);
        }
        if let Ok(mut entity_commands) = commands.get_entity(root) {
            entity_commands.remove::<PixelCameraChildren>();
        }
    }
}

pub(crate) fn cleanup_all_roots(
    mut commands: Commands,
    roots: Query<(Entity, &PixelCameraChildren), With<PixelCamera>>,
) {
    for (root, child_refs) in &roots {
        despawn_child_if_present(&mut commands, child_refs.inner);
        despawn_child_if_present(&mut commands, child_refs.canvas);
        despawn_child_if_present(&mut commands, child_refs.outer);
        if let Ok(mut entity_commands) = commands.get_entity(root) {
            entity_commands.remove::<PixelCameraChildren>();
        }
    }
}

pub(crate) fn ensure_pixel_camera_entities(
    mut commands: Commands,
    window_layout_version: Res<PixelCameraWindowLayoutVersion>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut scale_changes: MessageWriter<PixelScaleChanged>,
    mut roots: Query<(
        Entity,
        Ref<PixelCamera>,
        &mut PixelViewportMetrics,
        &mut PixelCameraWindowVersion,
        Option<&mut PixelCameraChildren>,
    )>,
    mut inner_cameras: Query<
        (
            &mut Camera,
            &mut Projection,
            &mut RenderTarget,
            &mut RenderLayers,
        ),
        (
            With<PixelCameraInner>,
            Without<PixelCameraCanvas>,
            Without<PixelCameraOuter>,
        ),
    >,
    mut canvas_sprites: Query<
        (&mut Sprite, &mut RenderLayers),
        (
            With<PixelCameraCanvas>,
            Without<PixelCameraInner>,
            Without<PixelCameraOuter>,
        ),
    >,
    mut outer_cameras: Query<
        (&mut Camera, &mut RenderLayers),
        (
            With<PixelCameraOuter>,
            Without<PixelCameraInner>,
            Without<PixelCameraCanvas>,
        ),
    >,
    helper_inner_cameras: Query<(), With<PixelCameraInner>>,
    helper_canvas_sprites: Query<(), With<PixelCameraCanvas>>,
    helper_outer_cameras: Query<(), With<PixelCameraOuter>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let window_info = primary_window_info(window);

    for (root, camera, mut runtime_metrics, mut seen_window_version, maybe_children) in &mut roots {
        if camera.world_layers.intersects(&camera.high_res_layers) {
            warn!(
                "pixel_camera root {root:?} has overlapping world/high-res render layers; pixel and HD content may double-render"
            );
        }

        let static_metrics = compute_static_metrics(&camera, window_info);
        let had_children = maybe_children.is_some();
        let helper_children = maybe_children.as_deref();
        let must_rebuild_helpers = helper_children.is_some_and(|children| {
            helpers_missing(
                children,
                &helper_inner_cameras,
                &helper_canvas_sprites,
                &helper_outer_cameras,
            )
        });
        let first_build = !had_children || must_rebuild_helpers;

        if first_build {
            if let Some(children) = helper_children {
                warn!(
                    "pixel_camera root {root:?} lost one or more helper entities; rebuilding inner camera, canvas, and outer camera"
                );
                cleanup_helper_entities(&mut commands, children);
            }

            let old_scale = runtime_metrics.integer_scale;
            let children = spawn_helpers(&mut commands, &mut images, root, &camera, static_metrics);
            apply_static_metrics(&mut runtime_metrics, static_metrics);
            seen_window_version.0 = window_layout_version.0;
            commands.entity(root).insert(children);
            if had_children && old_scale != 0 && old_scale != runtime_metrics.integer_scale {
                scale_changes.write(PixelScaleChanged {
                    camera: root,
                    old_scale,
                    new_scale: runtime_metrics.integer_scale,
                });
            }
            continue;
        }

        let needs_refresh = camera.is_added()
            || camera.is_changed()
            || seen_window_version.0 != window_layout_version.0
            || runtime_metrics.render_target_size != static_metrics.render_target_size
            || runtime_metrics.viewport_physical_size != static_metrics.viewport_physical_size
            || runtime_metrics.scale_factor != static_metrics.scale_factor;

        if !needs_refresh {
            continue;
        }

        let Some(mut children) = maybe_children else {
            continue;
        };
        let old_scale = runtime_metrics.integer_scale;

        let recreate_image = runtime_metrics.render_target_size
            != static_metrics.render_target_size
            || images.get(&children.image).is_none();
        if recreate_image {
            children.image =
                images.add(build_render_target_image(static_metrics.render_target_size));
        }

        reconfigure_inner_camera(
            &mut inner_cameras,
            children.inner,
            &camera,
            static_metrics,
            children.image.clone(),
        );
        reconfigure_canvas_sprite(
            &mut canvas_sprites,
            children.canvas,
            &camera,
            static_metrics,
            children.image.clone(),
        );
        reconfigure_outer_camera(&mut outer_cameras, children.outer, &camera);

        apply_static_metrics(&mut runtime_metrics, static_metrics);
        seen_window_version.0 = window_layout_version.0;

        if old_scale != 0 && old_scale != runtime_metrics.integer_scale {
            scale_changes.write(PixelScaleChanged {
                camera: root,
                old_scale,
                new_scale: runtime_metrics.integer_scale,
            });
        }
    }
}

pub(crate) fn refit_canvas_presentation(
    roots: Query<(&PixelViewportMetrics, &PixelCameraChildren), With<PixelCamera>>,
    mut canvases: Query<&mut Sprite, With<PixelCameraCanvas>>,
) {
    for (metrics, children) in &roots {
        let Ok(mut sprite) = canvases.get_mut(children.canvas) else {
            continue;
        };

        sprite.custom_size = Some(metrics.viewport_logical_size);
        let min = metrics.overscan.as_vec2() + metrics.texture_sample_offset;
        sprite.rect = Some(Rect::from_corners(
            min,
            min + metrics.virtual_size.as_vec2(),
        ));
    }
}

#[cfg(test)]
#[path = "canvas_tests.rs"]
mod tests;
