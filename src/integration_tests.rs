use bevy::{
    asset::AssetPlugin,
    ecs::{message::Messages, schedule::ScheduleLabel},
    prelude::*,
    window::{PrimaryWindow, WindowResized, WindowScaleFactorChanged},
};

use super::*;
use crate::components::PixelCameraChildren;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Activate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Deactivate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Tick;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Image>();
    app.add_message::<WindowResized>();
    app.add_message::<WindowScaleFactorChanged>();
    app.init_schedule(Activate);
    app.init_schedule(Deactivate);
    app.init_schedule(Tick);
    app.world_mut().spawn((
        Window {
            resolution: (1280, 720).into(),
            ..default()
        },
        PrimaryWindow,
    ));
    app.add_plugins(PixelCameraPlugin::new(Activate, Deactivate, Tick));
    app
}

fn boot(app: &mut App) {
    app.update();
}

fn activate_and_tick(app: &mut App) {
    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);
}

#[test]
fn plugin_supports_custom_schedules() {
    let mut app = test_app();
    boot(&mut app);
    let root = app.world_mut().spawn(PixelCamera::default()).id();

    app.update();
    assert!(app.world().get::<PixelCameraChildren>(root).is_none());

    app.world_mut().run_schedule(Activate);
    assert!(app.world().get::<PixelCameraChildren>(root).is_none());

    app.world_mut().run_schedule(Tick);
    assert!(app.world().get::<PixelCameraChildren>(root).is_some());
}

#[test]
fn plugin_spawns_expected_helper_entities() {
    let mut app = test_app();
    boot(&mut app);
    let root = app.world_mut().spawn(PixelCamera::default()).id();
    activate_and_tick(&mut app);

    let children = app
        .world()
        .get::<PixelCameraChildren>(root)
        .expect("helpers should be inserted");

    assert!(
        app.world()
            .get::<PixelCameraInner>(children.inner)
            .is_some()
    );
    assert!(
        app.world()
            .get::<PixelCameraCanvas>(children.canvas)
            .is_some()
    );
    assert!(
        app.world()
            .get::<PixelCameraOuter>(children.outer)
            .is_some()
    );
}

#[test]
fn deactivate_schedule_removes_helper_entities() {
    let mut app = test_app();
    boot(&mut app);
    let root = app.world_mut().spawn(PixelCamera::default()).id();
    activate_and_tick(&mut app);

    let children = app
        .world()
        .get::<PixelCameraChildren>(root)
        .expect("helpers should exist")
        .clone();

    app.world_mut().run_schedule(Deactivate);

    assert!(app.world().get_entity(children.inner).is_err());
    assert!(app.world().get_entity(children.canvas).is_err());
    assert!(app.world().get_entity(children.outer).is_err());
    assert!(app.world().get::<PixelCameraChildren>(root).is_none());
}

#[test]
fn removing_root_component_cleans_helper_entities() {
    let mut app = test_app();
    boot(&mut app);
    let root = app.world_mut().spawn(PixelCamera::default()).id();
    activate_and_tick(&mut app);

    let children = app
        .world()
        .get::<PixelCameraChildren>(root)
        .expect("helpers should exist")
        .clone();

    app.world_mut().entity_mut(root).remove::<PixelCamera>();
    app.world_mut().run_schedule(Tick);

    assert!(app.world().get_entity(children.inner).is_err());
    assert!(app.world().get_entity(children.canvas).is_err());
    assert!(app.world().get_entity(children.outer).is_err());
}

#[test]
fn config_change_updates_render_target_size() {
    let mut app = test_app();
    boot(&mut app);
    let root = app.world_mut().spawn(PixelCamera::default()).id();
    activate_and_tick(&mut app);

    app.world_mut().entity_mut(root).insert(PixelCamera {
        zoom: 3,
        ..PixelCamera::default()
    });
    app.world_mut().run_schedule(Tick);

    let metrics = app
        .world()
        .get::<PixelViewportMetrics>(root)
        .expect("metrics should exist");
    assert_eq!(metrics.render_target_size, UVec2::new(326, 186));
}

#[test]
fn always_on_plugin_works_with_update_schedule_only() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Image>();
    app.add_message::<WindowResized>();
    app.add_message::<WindowScaleFactorChanged>();
    app.world_mut().spawn((
        Window {
            resolution: (960, 540).into(),
            ..default()
        },
        PrimaryWindow,
    ));
    app.add_plugins(PixelCameraPlugin::always_on(Update));
    let root = app.world_mut().spawn(PixelCamera::default()).id();

    app.update();
    app.update();

    assert!(app.world().get::<PixelCameraChildren>(root).is_some());
}

#[test]
fn virtual_size_change_recreates_render_target_image() {
    let mut app = test_app();
    boot(&mut app);
    let root = app.world_mut().spawn(PixelCamera::default()).id();
    activate_and_tick(&mut app);

    let original_children = app
        .world()
        .get::<PixelCameraChildren>(root)
        .expect("helpers should exist")
        .clone();

    app.world_mut().entity_mut(root).insert(PixelCamera {
        virtual_size: UVec2::new(400, 240),
        ..PixelCamera::default()
    });
    app.world_mut().run_schedule(Tick);

    let children = app
        .world()
        .get::<PixelCameraChildren>(root)
        .expect("helpers should exist after virtual-size change");
    let metrics = app
        .world()
        .get::<PixelViewportMetrics>(root)
        .expect("metrics should exist after virtual-size change");

    assert_ne!(children.image, original_children.image);
    assert_eq!(metrics.virtual_size, UVec2::new(400, 240));
    assert_eq!(metrics.render_target_size, UVec2::new(402, 242));
}

#[test]
fn non_size_config_change_keeps_existing_render_target_image() {
    let mut app = test_app();
    boot(&mut app);
    let root = app.world_mut().spawn(PixelCamera::default()).id();
    activate_and_tick(&mut app);

    let original_children = app
        .world()
        .get::<PixelCameraChildren>(root)
        .expect("helpers should exist")
        .clone();

    app.world_mut().entity_mut(root).insert(PixelCamera {
        letterbox_color: Color::srgb(0.18, 0.10, 0.06),
        outer_camera_order: 5,
        ..PixelCamera::default()
    });
    app.world_mut().run_schedule(Tick);

    let children = app
        .world()
        .get::<PixelCameraChildren>(root)
        .expect("helpers should still exist");
    let outer_camera = app
        .world()
        .get::<Camera>(children.outer)
        .expect("outer camera should exist");

    assert_eq!(children.image, original_children.image);
    assert_eq!(outer_camera.order, 5);
}

#[test]
fn missing_helper_entities_are_rebuilt() {
    let mut app = test_app();
    boot(&mut app);
    let root = app.world_mut().spawn(PixelCamera::default()).id();
    activate_and_tick(&mut app);

    let original_children = app
        .world()
        .get::<PixelCameraChildren>(root)
        .expect("helpers should exist")
        .clone();

    app.world_mut()
        .entity_mut(original_children.canvas)
        .despawn();
    app.world_mut().run_schedule(Tick);

    let rebuilt_children = app
        .world()
        .get::<PixelCameraChildren>(root)
        .expect("helpers should be rebuilt")
        .clone();

    assert_ne!(rebuilt_children.inner, original_children.inner);
    assert_ne!(rebuilt_children.canvas, original_children.canvas);
    assert_ne!(rebuilt_children.outer, original_children.outer);
    assert!(
        app.world()
            .get::<PixelCameraInner>(rebuilt_children.inner)
            .is_some()
    );
    assert!(
        app.world()
            .get::<PixelCameraCanvas>(rebuilt_children.canvas)
            .is_some()
    );
    assert!(
        app.world()
            .get::<PixelCameraOuter>(rebuilt_children.outer)
            .is_some()
    );
    assert!(app.world().get_entity(original_children.inner).is_err());
    assert!(app.world().get_entity(original_children.canvas).is_err());
    assert!(app.world().get_entity(original_children.outer).is_err());
}

#[test]
fn scale_change_message_is_emitted_when_resize_changes_integer_scale() {
    let mut app = test_app();
    boot(&mut app);
    let root = app.world_mut().spawn(PixelCamera::default()).id();
    activate_and_tick(&mut app);

    {
        let world = app.world_mut();
        let mut query = world.query_filtered::<(Entity, &mut Window), With<PrimaryWindow>>();
        let (window_entity, mut window) = query.single_mut(world).expect("primary window exists");
        window.resolution.set(960.0, 540.0);
        world.write_message(WindowResized {
            window: window_entity,
            width: 960.0,
            height: 540.0,
        });
    }

    app.world_mut().run_schedule(Tick);

    let metrics = app
        .world()
        .get::<PixelViewportMetrics>(root)
        .expect("metrics should exist");
    assert_eq!(metrics.integer_scale, 3);
    assert!(
        !app.world()
            .resource::<Messages<PixelScaleChanged>>()
            .is_empty(),
        "expected a PixelScaleChanged message after integer scale changed"
    );
}
