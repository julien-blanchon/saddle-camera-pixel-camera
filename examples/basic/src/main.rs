use saddle_camera_pixel_camera_example_support as support;

use bevy::prelude::*;
use saddle_camera_pixel_camera::PixelCameraPlugin;
use support::{DemoActor, HighResBadge};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PixelCameraPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_actor, orbit_badge));
    support::maybe_install_auto_exit(&mut app);
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    support::spawn_demo_world(&mut commands, &mut images);
    support::spawn_overlay(&mut commands, "basic.rs\nPixel world + HD badge");
    support::spawn_pixel_camera_root(
        &mut commands,
        saddle_camera_pixel_camera::PixelCamera::default(),
        Vec2::ZERO,
    );
}

fn animate_actor(time: Res<Time>, mut actors: Query<&mut Transform, With<DemoActor>>) {
    for mut transform in &mut actors {
        let t = time.elapsed_secs();
        transform.translation.x = (t * 0.8).cos() * 42.0;
        transform.translation.y = (t * 1.6).sin() * 18.0;
    }
}

fn orbit_badge(time: Res<Time>, mut badges: Query<&mut Transform, With<HighResBadge>>) {
    for mut transform in &mut badges {
        let t = time.elapsed_secs();
        transform.translation.x = 280.0 + (t * 1.4).cos() * 28.0;
        transform.translation.y = 126.0 + (t * 1.4).sin() * 18.0;
    }
}
