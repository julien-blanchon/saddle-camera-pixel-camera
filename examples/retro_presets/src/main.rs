use saddle_camera_pixel_camera_example_support as support;

use bevy::prelude::*;
use saddle_camera_pixel_camera::{PixelCamera, PixelCameraPlugin};
use support::{DemoActor, ExamplePixelPane, OverlayText};

#[derive(Resource)]
struct RetroRoot(Entity);

#[derive(Resource, Default)]
struct ActivePreset(usize);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PixelCameraPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_actor, switch_presets, update_overlay));
    support::install_pane(&mut app);
    support::maybe_install_auto_exit(&mut app);
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let camera = PixelCamera::gba();
    let logical_position = Vec2::new(-16.0, 6.0);

    support::spawn_demo_world(&mut commands, &mut images);
    support::spawn_overlay(
        &mut commands,
        "retro_presets.rs\nPress 1 NES, 2 SNES, 3 Game Boy, 4 GBA",
    );
    let root = support::spawn_pixel_camera_root(&mut commands, camera.clone(), logical_position);
    commands.insert_resource(RetroRoot(root));
    commands.insert_resource(ActivePreset(3));
    support::queue_example_pane(
        &mut commands,
        ExamplePixelPane::from_setup(&camera, logical_position, None),
    );
}

fn animate_actor(time: Res<Time>, mut actors: Query<&mut Transform, With<DemoActor>>) {
    for mut transform in &mut actors {
        let t = time.elapsed_secs();
        transform.translation.x = (t * 1.0).cos() * 44.0;
        transform.translation.y = (t * 1.7).sin() * 20.0;
    }
}

fn switch_presets(
    keys: Res<ButtonInput<KeyCode>>,
    root: Res<RetroRoot>,
    mut active: ResMut<ActivePreset>,
    mut pane: ResMut<ExamplePixelPane>,
    mut cameras: Query<&mut PixelCamera>,
) {
    let selection = if keys.just_pressed(KeyCode::Digit1) {
        Some((0usize, PixelCamera::nes()))
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some((1usize, PixelCamera::snes()))
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some((2usize, PixelCamera::gameboy()))
    } else if keys.just_pressed(KeyCode::Digit4) {
        Some((3usize, PixelCamera::gba()))
    } else {
        None
    };

    let Some((index, preset)) = selection else {
        return;
    };
    let Ok(mut camera) = cameras.get_mut(root.0) else {
        return;
    };

    active.0 = index;
    pane.virtual_width = preset.virtual_size.x as f32;
    pane.virtual_height = preset.virtual_size.y as f32;
    pane.zoom = preset.zoom as f32;
    *camera = preset;
}

fn update_overlay(active: Res<ActivePreset>, mut text: Query<&mut Text, With<OverlayText>>) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };

    let label = match active.0 {
        0 => "NES 256x240",
        1 => "SNES 256x224",
        2 => "Game Boy 160x144",
        _ => "GBA 240x160",
    };
    text.0 = format!("retro_presets.rs\nactive preset: {label}\nPress 1/2/3/4 to switch");
}
