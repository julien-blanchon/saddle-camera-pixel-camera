use saddle_camera_pixel_camera_example_support as support;

use bevy::{prelude::*, window::PrimaryWindow};
use saddle_camera_pixel_camera::PixelCameraPlugin;

#[derive(Resource)]
struct ResizeCycle {
    timer: Timer,
    step: usize,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(ResizeCycle {
            timer: Timer::from_seconds(1.5, TimerMode::Repeating),
            step: 0,
        })
        .add_plugins(PixelCameraPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, cycle_window);
    support::install_pane(&mut app);
    support::maybe_install_auto_exit(&mut app);
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let camera = saddle_camera_pixel_camera::PixelCamera::default();
    support::spawn_demo_world(&mut commands, &mut images);
    support::spawn_overlay(&mut commands, "resize.rs\nCycles resolution + DPI override");
    support::spawn_pixel_camera_root(&mut commands, camera.clone(), Vec2::ZERO);
    support::queue_example_pane(
        &mut commands,
        support::ExamplePixelPane::from_setup(&camera, Vec2::ZERO, None),
    );
}

fn cycle_window(
    time: Res<Time>,
    mut cycle: ResMut<ResizeCycle>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    cycle.timer.tick(time.delta());
    if !cycle.timer.just_finished() {
        return;
    }

    let presets = [
        (1280.0, 720.0),
        (1600.0, 900.0),
        (1920.0, 1080.0),
        (1024.0, 768.0),
    ];
    let (width, height) = presets[cycle.step % presets.len()];
    window.resolution.set(width, height);
    if cycle.step.is_multiple_of(2) {
        window.resolution.set_scale_factor_override(Some(2.0));
    } else {
        window.resolution.set_scale_factor_override(None);
    }
    cycle.step += 1;
}
