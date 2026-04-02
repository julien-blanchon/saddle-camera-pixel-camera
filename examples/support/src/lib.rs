use bevy::{
    app::AppExit,
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use saddle_camera_pixel_camera::{HIGH_RES_LAYERS, PIXEL_LAYERS, PixelCamera, PixelCameraTransform};

pub const DEFAULT_VIRTUAL_SIZE: UVec2 = UVec2::new(320, 180);
pub const TILE_SIZE: f32 = 16.0;

#[derive(Resource)]
struct AutoExitAfter(Timer);

#[derive(Component)]
pub struct DemoActor;

#[derive(Component)]
pub struct HighResBadge;

#[derive(Component)]
pub struct CursorMarker;

#[derive(Component)]
pub struct OverlayText;

fn rgba_image(width: u32, height: u32, data: Vec<u8>) -> Image {
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
}

pub fn checker_image(size: u32, a: [u8; 4], b: [u8; 4]) -> Image {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let color = if (x + y) % 2 == 0 { a } else { b };
            data.extend_from_slice(&color);
        }
    }
    rgba_image(size, size, data)
}

pub fn actor_image() -> Image {
    let mut data = vec![0; 12 * 12 * 4];
    for y in 0..12 {
        for x in 0..12 {
            let idx = ((y * 12 + x) * 4) as usize;
            let color = if x == 0 || y == 0 || x == 11 || y == 11 {
                [24, 28, 36, 255]
            } else if (x == 3 || x == 8) && y == 4 {
                [255, 250, 240, 255]
            } else if y > 7 {
                [236, 111, 68, 255]
            } else {
                [93, 171, 243, 255]
            };
            data[idx..idx + 4].copy_from_slice(&color);
        }
    }
    rgba_image(12, 12, data)
}

pub fn badge_image() -> Image {
    let mut data = vec![0; 24 * 24 * 4];
    for y in 0..24 {
        for x in 0..24 {
            let idx = ((y * 24 + x) * 4) as usize;
            let dx = x as f32 - 11.5;
            let dy = y as f32 - 11.5;
            let distance = (dx * dx + dy * dy).sqrt();
            let color = if distance < 8.0 {
                [255, 208, 102, 255]
            } else if distance < 10.5 {
                [44, 58, 71, 255]
            } else {
                [0, 0, 0, 0]
            };
            data[idx..idx + 4].copy_from_slice(&color);
        }
    }
    rgba_image(24, 24, data)
}

pub fn cursor_image() -> Image {
    let mut data = vec![0; 8 * 8 * 4];
    for y in 0..8 {
        for x in 0..8 {
            let idx = ((y * 8 + x) * 4) as usize;
            let color = if x == 3 || x == 4 || y == 3 || y == 4 {
                [255, 245, 157, 255]
            } else {
                [0, 0, 0, 0]
            };
            data[idx..idx + 4].copy_from_slice(&color);
        }
    }
    rgba_image(8, 8, data)
}

pub fn spawn_demo_world(commands: &mut Commands, images: &mut Assets<Image>) -> Entity {
    let tile = images.add(checker_image(16, [72, 92, 105, 255], [57, 74, 86, 255]));
    let actor = images.add(actor_image());
    let badge = images.add(badge_image());

    for y in -8..=8 {
        for x in -12..=12 {
            commands.spawn((
                Name::new("Demo Tile"),
                Sprite {
                    image: tile.clone(),
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                Transform::from_xyz(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, -1.0),
                PIXEL_LAYERS.clone(),
            ));
        }
    }

    commands.spawn((
        Name::new("Pixel Parallax"),
        Sprite {
            image: tile,
            custom_size: Some(Vec2::new(420.0, 42.0)),
            color: Color::srgb_u8(36, 44, 60),
            ..default()
        },
        Transform::from_xyz(0.0, 82.0, -5.0),
        PIXEL_LAYERS.clone(),
    ));

    commands.spawn((
        Name::new("Demo Actor"),
        DemoActor,
        Sprite {
            image: actor,
            custom_size: Some(Vec2::splat(24.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 3.0),
        PIXEL_LAYERS.clone(),
    ));

    commands
        .spawn((
            Name::new("High Res Badge"),
            HighResBadge,
            Sprite {
                image: badge,
                custom_size: Some(Vec2::splat(72.0)),
                ..default()
            },
            Transform::from_xyz(290.0, 130.0, 10.0),
            HIGH_RES_LAYERS.clone(),
        ))
        .id()
}

pub fn spawn_overlay(commands: &mut Commands, label: &str) {
    commands.spawn((
        Name::new("Overlay Text"),
        OverlayText,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(18.0),
            left: Val::Px(18.0),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.07, 0.10, 0.78)),
        Text::new(label),
        TextFont {
            font_size: 17.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

pub fn spawn_cursor_marker(commands: &mut Commands, images: &mut Assets<Image>) -> Entity {
    let cursor = images.add(cursor_image());
    commands
        .spawn((
            Name::new("Cursor Marker"),
            CursorMarker,
            Sprite {
                image: cursor,
                custom_size: Some(Vec2::splat(8.0)),
                ..default()
            },
            Visibility::Hidden,
            Transform::from_xyz(0.0, 0.0, 4.0),
            PIXEL_LAYERS.clone(),
        ))
        .id()
}

pub fn spawn_pixel_camera_root(
    commands: &mut Commands,
    config: PixelCamera,
    logical_position: Vec2,
) -> Entity {
    commands
        .spawn((
            Name::new("Pixel Camera Root"),
            config,
            PixelCameraTransform { logical_position },
        ))
        .id()
}

pub fn maybe_install_auto_exit(app: &mut App) {
    let Some(seconds) = std::env::var("PIXEL_CAMERA_AUTO_EXIT_SECONDS")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
    else {
        return;
    };
    let seconds = seconds.max(0.1);

    // Graceful shutdown should happen through AppExit, but keep an env-only
    // fallback for batch verification where windowed examples can otherwise
    // outlive the expected timer under some host setups.
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs_f32(seconds + 0.5));
        std::process::exit(0);
    });

    app.insert_resource(AutoExitAfter(Timer::from_seconds(seconds, TimerMode::Once)));
    app.add_systems(Update, auto_exit_after);
}

fn auto_exit_after(
    time: Res<Time>,
    mut timer: ResMut<AutoExitAfter>,
    mut exit: MessageWriter<AppExit>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        exit.write(AppExit::Success);
    }
}
