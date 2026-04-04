use saddle_camera_pixel_camera_example_support as support;

use bevy::prelude::*;
use saddle_camera_pixel_camera::PixelCameraPlugin;
use support::HighResBadge;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PixelCameraPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, animate_badge);
    support::install_pane(&mut app);
    support::maybe_install_auto_exit(&mut app);
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let camera = saddle_camera_pixel_camera::PixelCamera::default();
    support::spawn_demo_world(&mut commands, &mut images);
    support::spawn_overlay(
        &mut commands,
        "mixed_layers.rs\nWorld = low-res, badge/UI = high-res",
    );
    support::spawn_pixel_camera_root(&mut commands, camera.clone(), Vec2::ZERO);
    support::queue_example_pane(
        &mut commands,
        support::ExamplePixelPane::from_setup(&camera, Vec2::ZERO, None),
    );
}

fn animate_badge(time: Res<Time>, mut badges: Query<&mut Transform, With<HighResBadge>>) {
    for mut transform in &mut badges {
        let t = time.elapsed_secs();
        transform.rotation = Quat::from_rotation_z((t * 0.8).sin() * 0.2);
        transform.translation.x = 280.0 + (t * 0.9).cos() * 18.0;
    }
}
