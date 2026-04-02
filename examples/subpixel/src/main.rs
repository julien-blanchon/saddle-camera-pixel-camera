use saddle_camera_pixel_camera_example_support as support;

use bevy::prelude::*;
use saddle_camera_pixel_camera::{PixelCamera, PixelCameraPlugin, PixelCameraTransform, PixelViewportMetrics};
use support::{DemoActor, OverlayText};

#[derive(Resource)]
struct PixelCameraRoot(Entity);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PixelCameraPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_actor, pan_camera, update_overlay));
    support::maybe_install_auto_exit(&mut app);
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    support::spawn_demo_world(&mut commands, &mut images);
    support::spawn_overlay(&mut commands, "subpixel.rs");
    let root = support::spawn_pixel_camera_root(
        &mut commands,
        PixelCamera::default(),
        Vec2::new(-24.25, 8.5),
    );
    commands.insert_resource(PixelCameraRoot(root));
}

fn animate_actor(time: Res<Time>, mut actors: Query<&mut Transform, With<DemoActor>>) {
    for mut transform in &mut actors {
        let t = time.elapsed_secs();
        transform.translation.x = (t * 1.2).sin() * 60.0;
        transform.translation.y = (t * 2.1).cos() * 24.0;
    }
}

fn pan_camera(
    time: Res<Time>,
    root: Res<PixelCameraRoot>,
    mut transforms: Query<&mut PixelCameraTransform>,
) {
    let Ok(mut transform) = transforms.get_mut(root.0) else {
        return;
    };
    let t = time.elapsed_secs();
    transform.logical_position = Vec2::new(t * 19.0 - 24.25, (t * 0.7).sin() * 10.0);
}

fn update_overlay(
    root: Res<PixelCameraRoot>,
    metrics: Query<&PixelViewportMetrics>,
    mut text: Query<&mut Text, With<OverlayText>>,
) {
    let Ok(metrics) = metrics.get(root.0) else {
        return;
    };
    let Ok(mut text) = text.single_mut() else {
        return;
    };

    text.0 = format!(
        "subpixel.rs\nscale={} snapped={:?}\nfractional={:.2}, {:.2}",
        metrics.integer_scale,
        metrics.snapped_position,
        metrics.fractional_offset.x,
        metrics.fractional_offset.y,
    );
}
