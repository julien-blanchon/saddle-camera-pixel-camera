use bevy::{
    app::AppExit,
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_flair::prelude::InlineStyle;
use saddle_camera_pixel_camera::{
    HIGH_RES_LAYERS, PIXEL_LAYERS, PixelCamera, PixelCameraScaleMode, PixelCameraTransform,
    PixelShake,
};
use saddle_pane::prelude::*;

const PANE_DARK_THEME_VARS: &[(&str, &str)] = &[
    ("--pane-elevation-1", "#28292e"),
    ("--pane-elevation-2", "#222327"),
    ("--pane-elevation-3", "rgba(187, 188, 196, 0.10)"),
    ("--pane-border", "#3c3d44"),
    ("--pane-border-focus", "#7090b0"),
    ("--pane-border-subtle", "#333438"),
    ("--pane-text-primary", "#bbbcc4"),
    ("--pane-text-secondary", "#78797f"),
    ("--pane-text-muted", "#5c5d64"),
    ("--pane-text-on-accent", "#ffffff"),
    ("--pane-text-brighter", "#d0d1d8"),
    ("--pane-text-monitor", "#9a9ba2"),
    ("--pane-text-log", "#8a8b92"),
    ("--pane-accent", "#4a6fa5"),
    ("--pane-accent-hover", "#5a8fd5"),
    ("--pane-accent-active", "#3a5f95"),
    ("--pane-accent-subtle", "rgba(74, 111, 165, 0.15)"),
    ("--pane-accent-fill", "rgba(74, 111, 165, 0.60)"),
    ("--pane-accent-fill-hover", "rgba(90, 143, 213, 0.70)"),
    ("--pane-accent-fill-active", "rgba(90, 143, 213, 0.80)"),
    ("--pane-accent-checked", "rgba(74, 111, 165, 0.25)"),
    ("--pane-accent-checked-hover", "rgba(74, 111, 165, 0.35)"),
    ("--pane-accent-indicator", "rgba(74, 111, 165, 0.80)"),
    ("--pane-accent-knob", "#7aacdf"),
    ("--pane-widget-bg", "rgba(187, 188, 196, 0.10)"),
    ("--pane-widget-hover", "rgba(187, 188, 196, 0.15)"),
    ("--pane-widget-focus", "rgba(187, 188, 196, 0.20)"),
    ("--pane-widget-active", "rgba(187, 188, 196, 0.25)"),
    ("--pane-widget-bg-muted", "rgba(187, 188, 196, 0.06)"),
    ("--pane-tab-hover-bg", "rgba(187, 188, 196, 0.06)"),
    ("--pane-hover-bg", "rgba(255, 255, 255, 0.03)"),
    ("--pane-active-bg", "rgba(255, 255, 255, 0.05)"),
    ("--pane-popup-bg", "#1e1f24"),
    ("--pane-bg-dark", "rgba(0, 0, 0, 0.25)"),
];

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

#[derive(Resource, Debug, Clone, Copy, PartialEq, Pane)]
#[pane(title = "Pixel Camera", position = "top-right")]
pub struct ExamplePixelPane {
    #[pane(slider, min = 120.0, max = 640.0, step = 1.0)]
    pub virtual_width: f32,
    #[pane(slider, min = 90.0, max = 360.0, step = 1.0)]
    pub virtual_height: f32,
    #[pane(slider, min = 1.0, max = 6.0, step = 1.0)]
    pub zoom: f32,
    #[pane(slider, min = 0.0, max = 2.0, step = 1.0)]
    pub scale_mode_index: f32,
    #[pane(slider, min = -180.0, max = 180.0, step = 0.5)]
    pub logical_x: f32,
    #[pane(slider, min = -120.0, max = 120.0, step = 0.5)]
    pub logical_y: f32,
    #[pane(slider, min = 0.0, max = 8.0, step = 0.1)]
    pub shake_amplitude: f32,
    #[pane(slider, min = 1.0, max = 24.0, step = 0.5)]
    pub shake_frequency: f32,
}

impl Default for ExamplePixelPane {
    fn default() -> Self {
        Self {
            virtual_width: DEFAULT_VIRTUAL_SIZE.x as f32,
            virtual_height: DEFAULT_VIRTUAL_SIZE.y as f32,
            zoom: 1.0,
            scale_mode_index: 0.0,
            logical_x: 0.0,
            logical_y: 0.0,
            shake_amplitude: 0.0,
            shake_frequency: 12.0,
        }
    }
}

impl ExamplePixelPane {
    pub fn from_setup(camera: &PixelCamera, transform: Vec2, shake: Option<&PixelShake>) -> Self {
        let scale_mode_index = match camera.scale_mode {
            PixelCameraScaleMode::IntegerLetterbox => 0.0,
            PixelCameraScaleMode::IntegerCrop => 1.0,
            PixelCameraScaleMode::FractionalFit => 2.0,
        };
        Self {
            virtual_width: camera.virtual_size.x as f32,
            virtual_height: camera.virtual_size.y as f32,
            zoom: camera.zoom as f32,
            scale_mode_index,
            logical_x: transform.x,
            logical_y: transform.y,
            shake_amplitude: shake.map_or(0.0, |shake| shake.amplitude),
            shake_frequency: shake.map_or(12.0, |shake| shake.frequency),
        }
    }
}

#[derive(Resource, Clone, Copy)]
struct ExamplePixelPaneBootstrap(ExamplePixelPane);

pub fn queue_example_pane(commands: &mut Commands, pane: ExamplePixelPane) {
    commands.insert_resource(ExamplePixelPaneBootstrap(pane));
}

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

pub fn install_pane(app: &mut App) {
    if !app.is_plugin_added::<PanePlugin>() {
        app.add_plugins((
            bevy_flair::FlairPlugin,
            bevy_input_focus::InputDispatchPlugin,
            bevy_ui_widgets::UiWidgetsPlugins,
            bevy_input_focus::tab_navigation::TabNavigationPlugin,
            PanePlugin,
        ));
    }

    app.register_pane::<ExamplePixelPane>()
        .add_systems(
            PreUpdate,
            (
                prime_pane_theme_vars,
                apply_bootstrapped_pane,
                sync_example_pane,
            )
                .chain(),
        )
        .add_systems(PostUpdate, reflect_example_pane);
}

fn prime_pane_theme_vars(mut panes: Query<&mut InlineStyle, Added<PaneRoot>>) {
    for mut style in &mut panes {
        for &(key, value) in PANE_DARK_THEME_VARS {
            style.set(key, value.to_owned());
        }
    }
}

fn apply_bootstrapped_pane(
    bootstrap: Option<Res<ExamplePixelPaneBootstrap>>,
    mut pane: ResMut<ExamplePixelPane>,
) {
    let Some(bootstrap) = bootstrap else {
        return;
    };

    if *pane == ExamplePixelPane::default() {
        *pane = bootstrap.0;
    }
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

fn sync_example_pane(
    mut pane: ResMut<ExamplePixelPane>,
    bootstrap: Option<Res<ExamplePixelPaneBootstrap>>,
    mut cameras: Query<(
        &mut PixelCamera,
        &mut PixelCameraTransform,
        Option<&mut PixelShake>,
    )>,
) {
    let has_bootstrap = bootstrap.is_some();
    if let Some(bootstrap) = bootstrap {
        if *pane == ExamplePixelPane::default() && bootstrap.0 != *pane {
            *pane = bootstrap.0;
        }
    }

    for (mut camera, mut transform, shake) in &mut cameras {
        let scene_pane =
            ExamplePixelPane::from_setup(&camera, transform.logical_position, shake.as_deref());
        if !has_bootstrap && *pane == ExamplePixelPane::default() && scene_pane != *pane {
            *pane = scene_pane;
            return;
        }

        camera.virtual_size = UVec2::new(
            pane.virtual_width.round().max(1.0) as u32,
            pane.virtual_height.round().max(1.0) as u32,
        );
        camera.zoom = pane.zoom.round().clamp(1.0, 6.0) as u32;
        camera.scale_mode = match pane.scale_mode_index.round() as i32 {
            1 => PixelCameraScaleMode::IntegerCrop,
            2 => PixelCameraScaleMode::FractionalFit,
            _ => PixelCameraScaleMode::IntegerLetterbox,
        };
        transform.logical_position = Vec2::new(pane.logical_x, pane.logical_y);

        if let Some(mut shake) = shake {
            shake.amplitude = pane.shake_amplitude.max(0.0);
            shake.frequency = pane.shake_frequency.max(0.1);
        }
    }
}

fn reflect_example_pane(
    mut pane: ResMut<ExamplePixelPane>,
    cameras: Query<(&PixelCamera, &PixelCameraTransform, Option<&PixelShake>)>,
) {
    let Some((camera, transform, shake)) = cameras.iter().next() else {
        return;
    };

    pane.virtual_width = camera.virtual_size.x as f32;
    pane.virtual_height = camera.virtual_size.y as f32;
    pane.zoom = camera.zoom as f32;
    pane.scale_mode_index = match camera.scale_mode {
        PixelCameraScaleMode::IntegerLetterbox => 0.0,
        PixelCameraScaleMode::IntegerCrop => 1.0,
        PixelCameraScaleMode::FractionalFit => 2.0,
    };
    pane.logical_x = transform.logical_position.x;
    pane.logical_y = transform.logical_position.y;
    pane.shake_amplitude = shake.map_or(0.0, |shake| shake.amplitude);
    pane.shake_frequency = shake.map_or(12.0, |shake| shake.frequency);
}
