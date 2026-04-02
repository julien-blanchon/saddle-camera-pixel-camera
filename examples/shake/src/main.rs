use saddle_camera_pixel_camera_example_support as support;

use bevy::prelude::*;
use saddle_camera_pixel_camera::{PixelCamera, PixelCameraPlugin, PixelShake};

#[derive(Resource)]
struct PulseTimer(Timer);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(PulseTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .add_plugins(PixelCameraPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, pulse_shake);
    support::maybe_install_auto_exit(&mut app);
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    support::spawn_demo_world(&mut commands, &mut images);
    support::spawn_overlay(
        &mut commands,
        "shake.rs\nCanvas shake stays on the pixel grid",
    );
    commands.spawn((
        Name::new("Shake Camera Root"),
        PixelCamera::default(),
        saddle_camera_pixel_camera::PixelCameraTransform::default(),
        PixelShake {
            amplitude: 0.0,
            frequency: 16.0,
            decay: 10.0,
            seed: 7,
        },
    ));
}

fn pulse_shake(time: Res<Time>, mut timer: ResMut<PulseTimer>, mut shakes: Query<&mut PixelShake>) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    for mut shake in &mut shakes {
        shake.amplitude = 3.0;
    }
}
